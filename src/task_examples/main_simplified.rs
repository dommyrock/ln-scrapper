use headless_chrome::Browser;
use rand::Rng;
use std::sync::{Arc, Mutex};
use tokio::sync::Semaphore;

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
struct Job {
    url: String,
    body: String,
    salary: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let semaphore = Arc::new(Semaphore::new(4)); // max 4 tasks
    let jobs = Arc::new(std::sync::RwLock::new(Vec::new()));
    let contents = std::fs::read_to_string("small_json_test.csv")?;
    let urls: Vec<&str> = contents.split(",").collect();
    let mut handles: Vec<tokio::task::JoinHandle<()>> = vec![];

    //allows Us to share the browser across multiple tasks.
    let browser = Arc::new(Mutex::new(Browser::default().unwrap()));

    urls.into_iter().for_each(|url| {
        let sem_clone = Arc::clone(&semaphore);
        let browser = Arc::clone(&browser);
        let url = url.to_owned();
        let random_delay: u64 = rand::thread_rng().gen_range(50..=80) + 200;
        let jobs = Arc::clone(&jobs);

        handles.push(tokio::spawn(async move {
            let permit = sem_clone.clone().acquire_owned().await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(random_delay)).await;

            if let Ok(page) = browser.lock().unwrap().new_tab() {
                if let Ok(tab) = page.navigate_to(&url) {
                    if tab.wait_for_element(".show-more-less-html__markup").is_ok() {
                        if let Ok(element) = tab.find_element(".show-more-less-html__markup") {
                            println!("{}", &url);
                            let content = element.get_content().unwrap();
                            println!("{}", content);

                            let salary: Option<String> = content
                                .find("Salary:")
                                .map(|index| content[index..].to_string());

                            let job = Job {
                                url: url.clone(),
                                body: content,
                                salary,
                            };

                            let mut jobs_write = jobs.write().unwrap();
                            jobs_write.push(job);
                        }
                    }
                }
            }

            //A JoinHandle detaches the associated task when it is dropped, which means that there is no longer any handle to the task, and no way to join on it.
            drop(permit);
        }));
    });

    for handle in handles {
        println!("Awaiting for all tasks to complete ...");
        handle.await.unwrap();
    }

    let file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open("export_jobs.json")
        .unwrap();

    let json_out = jobs.read().unwrap();
    serde_json::to_writer(file, &*json_out)
        .map(|_| println!("Outputed Jobs to export_jobs.json"))
        .expect("Error writing to file");

    Ok(())
}