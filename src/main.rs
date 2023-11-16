use headless_chrome::{Browser, LaunchOptionsBuilder};
use rand::Rng;
use std::collections::VecDeque;
use std::error::Error;
use std::sync::{Arc, Mutex};
use tokio::sync::Semaphore;
use tokio::task::{JoinError, JoinSet};
use tokio::time::Duration;
use urlencoding::decode;

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
struct Job {
    url: String,
    body: String,
    salary: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let semaphore = Arc::new(Semaphore::new(4));
    let jobs = Arc::new(std::sync::RwLock::new(Vec::new()));
    let contents = std::fs::read_to_string("small_json_test.csv")?;
    let urls: Vec<&str> = contents.split(",").collect();

    let decoded_urls = urls
    .into_iter()
    .map(|u| decode(u).expect("UTF-8"))
    .map(|f| f.split("&trackingId").collect::<Vec<&str>>()[0].trim_end_matches("=").to_owned())
    .collect::<Vec<String>>();

    //allows Us to share the browser across multiple tasks.
    // let browser = Arc::new(Mutex::new(Browser::default().unwrap()));//default
    let options = LaunchOptionsBuilder::default()
        .headless(false)
        .build()
        .unwrap();
    let browser = Arc::new(Mutex::new(Browser::new(options).unwrap()));

    let url_queue = Mutex::new(VecDeque::<String>::from(decoded_urls));

    loop {
        match url_queue.lock().unwrap().pop_back() {
            Some(url) => {
                let sem_clone = Arc::clone(&semaphore);
                let url = url.to_owned();
                let browser = Arc::clone(&browser);
                //This will block until one of the permits is available.
                let _permit = sem_clone.acquire().await.unwrap();
                let random_delay: u64 = rand::thread_rng().gen_range(50..=80) + 200;
                let jobs_ptr = Arc::clone(&jobs);

                //DO work
                tokio::spawn(async move {
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
                                        url: url.to_string(),
                                        body: content,
                                        salary,
                                    };

                                    let mut jobs_write = jobs_ptr.write().unwrap();
                                    jobs_write.push(job);
                                })
                                .unwrap_or_else(|e| {
                                    println!("Error finding element on {}: {}", &url, e)
                                });
                        }
                        let _ = page.close_target();
                    }
                });
                drop(_permit);
            }
            None => {
                println!("NO MORE ITEMS IN QUEUE: EXITING ...");
                break;
            }
        }
    }

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

async fn _unused_handle_task_results(mut tasks: JoinSet<Result<(), JoinError>>) {
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
