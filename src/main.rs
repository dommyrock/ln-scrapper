use headless_chrome::Browser;
use rand::Rng;
use std::error::Error;
use std::sync::{Arc, Mutex};
use tokio::sync::Semaphore;
use tokio::task::{JoinError, JoinSet};
use tokio::time::{sleep, Duration};

//TODO : Analyze code to check if semaphores worka as intended
//Right now it seems like we have big pause between initial 4 tasks in semaphore

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Create a semaphore maximum of 4 concurrent tasks
    let semaphore = Arc::new(Semaphore::new(4));
    let mut tasks: JoinSet<Result<(), JoinError>> = JoinSet::new();

    let contents = std::fs::read_to_string(
        "C:\\Users\\dpolzer\\Me\\Git\\ln-scrapper\\Demo_Sucess_830_jobs.csv",
    )?;
    let urls: Vec<&str> = contents.split(",").collect();

    //allows Us to share the browser across multiple tasks.
    let browser = Arc::new(Mutex::new(Browser::default().unwrap()));

    urls.into_iter().for_each(|url| {
        let sem_clone = Arc::clone(&semaphore);
        let url = url.to_owned();
        let browser = Arc::clone(&browser);
        let random_delay: u64 = rand::thread_rng().gen_range(50..=80) + 500;

        let task = tokio::spawn(async move {
            sleep(tokio::time::Duration::from_millis(random_delay)).await;

            let _permit = sem_clone.acquire().await.unwrap();

            println!("Task started: delay [{}]\n {}", &url, random_delay);

            if let Ok(page) = browser.lock().unwrap().new_tab() {
                if let Ok(tab) = page.navigate_to(&url) {
                    if tab.wait_for_element(".show-more-less-html__markup").is_ok() {
                        if let Ok(element) = tab.find_element(".show-more-less-html__markup") {
                            println!("{}", &url);
                            let content = element.get_content().unwrap();

                        }
                    }
                }
            }
        });

        tasks.spawn(task);
    });

    handle_task_results(tasks).await;

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

//v2
// use tokio::sync::Semaphore;
// use std::sync::Arc;

// #[tokio::main]
// async fn main() {
//     let semaphore = Arc::new(Semaphore::new(4));  // max 4 tasks

//     let mut handles = vec![];

//     for i in 0..10 {  // assuming we have 10 tasks
//         let permit = semaphore.clone().acquire_owned().await.unwrap();
//         let handle = tokio::spawn(async move {
//             println!("Task {} started", i);
//             tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;  // simulate task work
//             println!("Task {} ended", i);
//             drop(permit);
//         });

//         handles.push(handle);
//     }

//     // Wait for all tasks to complete
//     for handle in handles {
//         handle.await.unwrap();
//     }
// }
