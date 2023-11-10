use headless_chrome::{Browser, LaunchOptionsBuilder};
use rand::Rng;
use std::error::Error;
use std::sync::{Arc, Mutex};
use tokio::sync::Semaphore;
use tokio::task::{JoinError, JoinSet};
use tokio::time::{sleep, Duration};

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
struct Job {
    url: String,
    body: String,
    salary: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Create a semaphore maximum of 4 concurrent tasks
    let semaphore = Arc::new(Semaphore::new(4));
    let mut tasks: JoinSet<Result<(), JoinError>> = JoinSet::new();
    let jobs = Arc::new(std::sync::RwLock::new(Vec::new()));
    let contents = std::fs::read_to_string("small_json_test.csv")?;
    let urls: Vec<&str> = contents.split(",").collect();

    //allows Us to share the browser across multiple tasks.
    // let browser = Arc::new(Mutex::new(Browser::default().unwrap()));//default
    let options = LaunchOptionsBuilder::default()
        .headless(false)
        .build()
        .unwrap();
    let browser = Arc::new(Mutex::new(Browser::new(options).unwrap()));

    urls.into_iter().for_each(|url| {
        let sem_clone = Arc::clone(&semaphore);
        let url = url.to_owned();
        let browser = Arc::clone(&browser);
        let random_delay: u64 = rand::thread_rng().gen_range(50..=80) + 1000;
        let jobs_ptr = Arc::clone(&jobs);

        let task = tokio::spawn(async move {
            let _permit = sem_clone.acquire().await.unwrap();

            if let Ok(page) = browser.lock().unwrap().new_tab() {
                if let Ok(tab) = page.navigate_to(&url) {
                    println!("URL {}\nDelay {} ms", &url, random_delay);

                    std::thread::sleep(Duration::from_millis(random_delay));

                    tab.find_element(".show-more-less-html__markup")
                        .map(|elm| {
                            println!("Found element");
                            let content = elm.get_content().unwrap();
                            println!("{}", content);

                            let salary: Option<String> = content
                                .find("Salary:")
                                .map(|index| content[index..].to_string());

                            let job = Job {
                                url: url.clone(),
                                body: content,
                                salary,
                            };

                            let mut jobs_write = jobs_ptr.write().unwrap();
                            jobs_write.push(job);
                        })
                        .expect("Failed to find element on");
                }
            }
        });
        // does it need to be here or after the loop? Since idea is to iterate over task list. and then wait for all tasks to complete.
        tasks.spawn(task);

    });

    handle_task_results(tasks).await;

    let file = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open("export_jobs.json")
        .unwrap();

    let json_out = jobs.read().unwrap();
    serde_json::to_writer(file, &*json_out)
        .map(|_| println!("Outputed Jobs to export_jobs.json"))
        .expect("Error writing to file");

    Ok(())
}

async fn handle_task_results(mut tasks: JoinSet<Result<(), JoinError>>) {
    println!("Waiting for all tasks to complete ...\n");
    while let Some(res) = tasks.join_next().await {
        match res {
            Ok(Ok(_)) => {
                // The task completed successfully
            }
            Ok(Err(e)) => {
                // The task returned an error
                eprintln!("Task returned an error: {:?}", e);
            }
            Err(e) => {
                // The task was cancelled
                eprintln!("Task was cancelled: {:?}", e);
            }
        }
    }
}
