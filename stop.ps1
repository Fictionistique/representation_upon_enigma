# Stop all services
Write-Host "🛑 Stopping services..." -ForegroundColor Yellow
docker-compose down

Write-Host "✓ Services stopped" -ForegroundColor Green
Write-Host "`nTo restart services, run: docker-compose up -d" -ForegroundColor Cyan

