import puppeteer from "puppeteer"; //"^21.4.1"
import fs from "fs";

(async () => {
  console.log("Starting scrapper");

  const browser = await puppeteer.launch({ headless: false, channel: "chrome" });
  const page = await browser.newPage(); //aka tab
  let BASE_URL =
    "https://www.linkedin.com/jobs/search/?currentJobId=3744786882&f_C=69929&geoId=92000000&origin=COMPANY_PAGE_JOBS_CLUSTER_EXPANSION&originToLandingJobPostings=3749576450%2C3748589075%2C3748583737%2C3748585458%2C3749574738%2C3749545008%2C3748584515%2C3748734107%2C3748582681&start=0";
  await page.goto(BASE_URL);

  //block for 250ms
  await new Promise((r) => setTimeout(r, 250));

  let currentCount = 0;
  let results = {};
  while (true) {
    console.log("Scrolling again");
    // Scroll and check for button
    await page.evaluate(async () => {
      const maxScrollTime = 8000; //Optimal time to wait between button renders / scrolling
      const startTime = Date.now();
      const distance = 100;

      await new Promise((resolve) => {
        var timer = setInterval(() => {
          window.scrollBy(0, distance);
          let randomDelay = Math.floor(Math.random() * (650 - 450 + 1)) + 450; 
          // Check for button after each scroll
          const button = document.querySelector('button[aria-label="See more jobs"]');
          if (button) {
            //wait 600ms between each button click to avoid being blocked
            setTimeout(async () => {
              await button.click();
            }, randomDelay);
       
            // button.click();
          }

          if (Date.now() - startTime > maxScrollTime) {
            clearInterval(timer);
            resolve();
          }
        }, 100);
      });
    });

    const data = await page.evaluate(() => {
      const parent = document.querySelector("ul.jobs-search__results-list");
      const children = Array.from(parent.querySelectorAll("li a.base-card__full-link"));
      const c_len = children.length;
      return { hrefs: children.map((child) => child.href).join(","), count: c_len };
    });

    //if 'currentCount' is the same as the last check, it means we scrolled to the bottom
    if (data.count === currentCount) break;

    currentCount = data.count;
    results = data;
  }
  console.warn("DONE scrolling ------------------ Printing to file ");

  console.log("Found total of [" + results.count + "] links");
  console.log("writing contents to file" + "jobs.csv");

  await fs.writeFileSync("jobs.csv", results.hrefs);

  //Give browser time to close so i have time to write to file
  // await new Promise((r) => setTimeout(r, 3500));

  //remove loop once this works
  await browser.close();
})();
