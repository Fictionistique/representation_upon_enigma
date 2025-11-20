# Quick run script for common operations
param(
    [Parameter(Mandatory=$true, Position=0)]
    [ValidateSet('ingest', 'query', 'init')]
    [string]$Command,
    
    [Parameter(Mandatory=$false, Position=1)]
    [string]$Query = "",
    
    [int]$Count = 3,
    [int]$Limit = 3
)

switch ($Command) {
    'init' {
        Write-Host "🔧 Initializing vector database..." -ForegroundColor Cyan
        cargo run -- init
    }
    'ingest' {
        Write-Host "📥 Ingesting $Count bills..." -ForegroundColor Cyan
        cargo run -- ingest --count $Count
    }
    'query' {
        if ([string]::IsNullOrWhiteSpace($Query)) {
            Write-Host "❌ Please provide a query string" -ForegroundColor Red
            Write-Host "Example: .\run.ps1 query 'What are data protection rights?'" -ForegroundColor Yellow
            exit 1
        }
        Write-Host "🔍 Searching: $Query" -ForegroundColor Cyan
        cargo run -- query "$Query" --limit $Limit
    }
}

