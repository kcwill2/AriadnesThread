$ErrorActionPreference = "Stop"

$exe = "C:\Users\Kurt\AppData\Roaming\npm\node_modules\@opencode\cli\bin\opencode.exe"
$cwd = "C:\Users\Kurt\Code\AriadnesThread"

$p = New-Object System.Diagnostics.Process
$p.StartInfo = New-Object System.Diagnostics.ProcessStartInfo
$p.StartInfo.FileName = $exe
$p.StartInfo.Arguments = "acp"
$p.StartInfo.WorkingDirectory = $cwd
$p.StartInfo.UseShellExecute = $false
$p.StartInfo.RedirectStandardInput = $true
$p.StartInfo.RedirectStandardOutput = $true
$p.StartInfo.RedirectStandardError = $true
$p.StartInfo.CreateNoWindow = $true

$p.Start() | Out-Null

$input = $p.StandardInput
$output = $p.StandardOutput

function Send-Acp($message) {
    $json = $message | ConvertTo-Json -Compress -Depth 20
    Write-Host ">>> $json"
    $input.WriteLine($json)
    $input.Flush()
}

# ACP initialize
Send-Acp @{
    jsonrpc = "2.0"
    id = 1
    method = "initialize"
    params = @{
        protocolVersion = 1
        clientCapabilities = @{}
        clientInfo = @{
            name = "manual-test"
            version = "1.0.0"
        }
    }
}

Start-Sleep -Seconds 2

# Create session
Send-Acp @{
    jsonrpc = "2.0"
    id = 2
    method = "session/new"
    params = @{
        cwd = $cwd
        mcpServers = @()
    }
}

Start-Sleep -Seconds 2

Write-Host ""
Write-Host "Reading responses..."
Write-Host "Press Ctrl+C if nothing happens after ~10 seconds."
Write-Host ""

while (-not $output.EndOfStream) {
    $line = $output.ReadLine()
    if ($line) {
        Write-Host "<<< $line"
    }
}
