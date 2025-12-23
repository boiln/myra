# Handles formatting for JavaScript/TypeScript/CSS/HTML and Rust code

Write-Host "🚀 Starting project formatting..." -ForegroundColor Cyan

# Step 1: Format JS/TS files with Prettier
Write-Host "🎨 Formatting JavaScript/TypeScript files..." -ForegroundColor Yellow
pnpm fmt:js
if ($LASTEXITCODE -ne 0) {
    Write-Host "⚠️ Prettier formatting had issues, but continuing..." -ForegroundColor Yellow
}

# Step 2: Run ESLint to fix linting issues
Write-Host "🔍 Running ESLint to fix issues..." -ForegroundColor Yellow
pnpm lint
if ($LASTEXITCODE -ne 0) {
    Write-Host "⚠️ ESLint had issues, but continuing..." -ForegroundColor Yellow
}

# Step 3: Format Rust files
Write-Host "🦀 Formatting Rust code in src-tauri..." -ForegroundColor Yellow
cd src-tauri
cargo fmt --all
if ($LASTEXITCODE -ne 0) {
    Write-Host "⚠️ Rust formatting had issues, but continuing..." -ForegroundColor Yellow
}

# Step 4: Run Clippy to fix Rust code issues
Write-Host "🔧 Running Clippy to fix Rust issues..." -ForegroundColor Yellow
cargo clippy --fix --allow-dirty --allow-no-vcs
if ($LASTEXITCODE -ne 0) {
    Write-Host "⚠️ Clippy had issues, but continuing..." -ForegroundColor Yellow
}

cd ..
Write-Host "✅ Formatting complete!" -ForegroundColor Green 