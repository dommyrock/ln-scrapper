$data = Get-Content -Path '.\small_json_test.csv'
$urls = $data -split ','
$totalItems = $urls.Count
Write-Host "Opening total of ${script:totalItems} snippet images" -ForegroundColor Blue

try {
   foreach ($url in $urls) {
   
      # Ignore empty lines
      if (![string]::IsNullOrWhiteSpace($url)) {
         # Open the URL in a new browser tab
         [system.Diagnostics.Process]::Start("chrome", $url)
      }
   }
}
finally {
   Write-Host "opened total of ${script:totalItems} snippet images in: $($Stopwatch.Elapsed)" -ForegroundColor Green
}