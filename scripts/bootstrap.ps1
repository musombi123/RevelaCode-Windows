$ErrorActionPreference = "Stop"

$Root = Split-Path -Parent $PSScriptRoot
$Workspace = Join-Path $Root "workspace"
New-Item -ItemType Directory -Force -Path $Workspace | Out-Null

$requiredCommands = @("git", "node", "npm", "rustc", "cargo")
foreach ($commandName in $requiredCommands) {
    if (-not (Get-Command $commandName -ErrorAction SilentlyContinue)) {
        throw "Required command '$commandName' was not found. Install it and reopen PowerShell."
    }
}

$nodeVersion = (node --version).Trim()
if ($nodeVersion -notmatch '^v(22|23|24)\.') {
    throw "Node.js 22 or newer LTS is required. Found $nodeVersion."
}

$repositories = @(
    @{ Name = "frontend"; Url = "https://github.com/musombi123/RevelaCode-Frontend.git"; Branch = "main" },
    @{ Name = "backend"; Url = "https://github.com/musombi123/RevelaCode-Backend.git"; Branch = "main" },
    @{ Name = "revelaai"; Url = "https://github.com/musombi123/RevelaAI.git"; Branch = "main" }
)

foreach ($repository in $repositories) {
    $destination = Join-Path $Workspace $repository.Name
    if (-not (Test-Path (Join-Path $destination ".git"))) {
        if (Test-Path $destination) {
            throw "Destination exists but is not a Git checkout: $destination"
        }
        git clone --branch $repository.Branch --single-branch $repository.Url $destination
        if ($LASTEXITCODE -ne 0) { throw "Failed to clone $($repository.Name)." }
    } else {
        git -C $destination fetch origin
        if ($LASTEXITCODE -ne 0) { throw "Failed to fetch $($repository.Name)." }
        git -C $destination checkout $repository.Branch
        if ($LASTEXITCODE -ne 0) { throw "Failed to checkout $($repository.Branch) in $($repository.Name)." }
        git -C $destination pull --ff-only origin $repository.Branch
        if ($LASTEXITCODE -ne 0) { throw "Failed to fast-forward $($repository.Name)." }
    }
}

Push-Location $Root
try {
    npm install
    if ($LASTEXITCODE -ne 0) { throw "Failed to install the Windows host dependencies." }

    npm --prefix (Join-Path $Workspace "frontend") install
    if ($LASTEXITCODE -ne 0) { throw "Failed to install the frontend dependencies." }

    node (Join-Path $Root "scripts/verify-workspace.mjs")
    if ($LASTEXITCODE -ne 0) { throw "Workspace verification failed." }
} finally {
    Pop-Location
}

Write-Host ""
Write-Host "RevelaCode Windows workspace is ready." -ForegroundColor Green
Write-Host "Run 'npm run tauri dev' from the repository root to launch the desktop app."
