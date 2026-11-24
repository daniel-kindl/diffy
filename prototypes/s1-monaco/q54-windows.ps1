# Throwaway Q54 run on the Windows notebook (map decision Q12). Never merged.
# Needs Node.js 24, Rust stable (MSVC), Git for Windows and WebView2 (part of Windows 11).
# Run from this folder in PowerShell:  .\q54-windows.ps1
# Do not use the keyboard or the mouse while the prototype window is open.
$ErrorActionPreference = "Stop"
Set-Location $PSScriptRoot

npm ci
npm run fixtures
npx tauri build --no-bundle
$bin = "src-tauri\target\release\diffy-s1-monaco.exe"

function Run($vars) {
  foreach ($k in $vars.Keys) { Set-Item "env:$k" $vars[$k] }
  & $bin | Out-Host
  foreach ($k in $vars.Keys) { Remove-Item "env:$k" }
}

# The checks: coordinates, findings, scrolling, live refresh and the horizontal scroll bar.
foreach ($o in "A", "B") {
  Run @{ DIFFY_S1_Q54 = $o; DIFFY_S1_Q54_PARTS = "coord+findings+scroll+live+hscroll"; DIFFY_S1_Q54_FIXTURES = "small,1mb,10mb,10k-lines"; DIFFY_S1_Q54_LABEL = "q54-windows-checks-$o" }
}
# The open times: 21 runs, run 1 is a warm-up.
Run @{ DIFFY_S1_AUTORUN = "A:1mb,A:10mb,B:1mb,B:10mb"; DIFFY_S1_RUNS = "21"; DIFFY_S1_LABEL = "q54-windows-open" }

Get-ChildItem bench-results\q54-windows-*.json | ForEach-Object {
  $r = Get-Content $_ -Raw | ConvertFrom-Json
  if ($r.records) { "{0}: passed {1}, failed: {2}" -f $_.Name, $r.passed, ($r.failed -join ", ") }
  else { "{0}: written" -f $_.Name }
}
"Send the files in bench-results\ whose names start with q54-windows-."
