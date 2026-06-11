# Install Galene for ClassMate live video (Windows)
$ErrorActionPreference = "Stop"

$appData = [Environment]::GetFolderPath("ApplicationData")
$targetDir = Join-Path $appData "com.classmate.app\galene"
New-Item -ItemType Directory -Force -Path $targetDir | Out-Null

$releaseUrl = "https://github.com/jech/galene/releases/latest/download/galene-windows-amd64.exe"
$outFile = Join-Path $targetDir "galene.exe"

Write-Host "Downloading Galene to $outFile ..."
Invoke-WebRequest -Uri $releaseUrl -OutFile $outFile -UseBasicParsing

Write-Host "Done. Start Class Hub with video enabled in ClassMate Settings or Class Hub."
Write-Host "Galene teacher login: teacher / classmate"
