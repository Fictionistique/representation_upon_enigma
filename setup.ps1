# Setup script for Civic Legislation Platform
Write-Host "🏛️  Civic Legislation Platform - Setup" -ForegroundColor Cyan
Write-Host "======================================`n" -ForegroundColor Cyan

# Check if Docker is running
Write-Host "Checking Docker..." -ForegroundColor Yellow
try {
    docker ps | Out-Null
    Write-Host "✓ Docker is running" -ForegroundColor Green
} catch {
    Write-Host "✗ Docker is not running. Please start Docker Desktop." -ForegroundColor Red
    exit 1
}

# Start services
Write-Host "`nStarting services (Qdrant, PostgreSQL)..." -ForegroundColor Yellow
docker-compose up -d

# Wait for services to be healthy
Write-Host "Waiting for services to be ready..." -ForegroundColor Yellow
Start-Sleep -Seconds 5

# Check service health
Write-Host "`nChecking service health..." -ForegroundColor Yellow
$qdrantHealth = docker inspect --format='{{.State.Health.Status}}' civic_qdrant 2>$null
$postgresHealth = docker inspect --format='{{.State.Health.Status}}' civic_postgres 2>$null

if ($qdrantHealth -eq "healthy" -or $qdrantHealth -eq $null) {
    Write-Host "✓ Qdrant is running at http://localhost:6333" -ForegroundColor Green
} else {
    Write-Host "⚠ Qdrant starting... (Status: $qdrantHealth)" -ForegroundColor Yellow
}

if ($postgresHealth -eq "healthy" -or $postgresHealth -eq $null) {
    Write-Host "✓ PostgreSQL is running at localhost:5432" -ForegroundColor Green
} else {
    Write-Host "⚠ PostgreSQL starting... (Status: $postgresHealth)" -ForegroundColor Yellow
}

# Build the project
Write-Host "`nBuilding Rust project..." -ForegroundColor Yellow
cargo build --release
if ($LASTEXITCODE -eq 0) {
    Write-Host "✓ Build successful" -ForegroundColor Green
} else {
    Write-Host "✗ Build failed" -ForegroundColor Red
    exit 1
}

# Initialize vector database
Write-Host "`nInitializing vector database..." -ForegroundColor Yellow
cargo run --release -- init
if ($LASTEXITCODE -eq 0) {
    Write-Host "✓ Vector database initialized" -ForegroundColor Green
} else {
    Write-Host "✗ Initialization failed" -ForegroundColor Red
    exit 1
}

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "✓ Setup completed successfully!" -ForegroundColor Green
Write-Host "`nNext steps:" -ForegroundColor Cyan
Write-Host "  1. Ingest bills:  .\run.ps1 ingest" -ForegroundColor White
Write-Host "  2. Query:         .\run.ps1 query 'your question'" -ForegroundColor White
Write-Host "`nOr use cargo directly:" -ForegroundColor Cyan
Write-Host "  cargo run -- ingest --count 3" -ForegroundColor White
Write-Host "  cargo run -- query 'What are data protection rights?'" -ForegroundColor White
Write-Host "`nℹ️  Using Candle + BERT for semantic search" -ForegroundColor Cyan
Write-Host "ℹ️  First run downloads model (~90MB) from HuggingFace" -ForegroundColor Cyan
Write-Host "========================================`n" -ForegroundColor Cyan

