param(
  [switch]$FullBuild,
  [string]$ReportPath
)

$ErrorActionPreference = "Continue"
$root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$checks = New-Object System.Collections.Generic.List[object]

function Add-Check {
  param(
    [string]$Name,
    [string]$Status,
    [string]$Details
  )

  $checks.Add([PSCustomObject]@{
    Name = $Name
    Status = $Status
    Details = $Details
  }) | Out-Null
}

function Invoke-Check {
  param(
    [string]$Name,
    [string]$CommandLine,
    [string]$WorkingDirectory = $root
  )

  Write-Host "== $Name"
  Push-Location $WorkingDirectory
  try {
    $global:LASTEXITCODE = 0
    $output = & cmd.exe /d /s /c "$CommandLine 2>&1" | Out-String
    $exitCode = $LASTEXITCODE
    $escape = [char]27
    $output = $output -replace "$escape\[[0-9;]*m", ""

    if ($exitCode -eq 0) {
      Add-Check $Name "pass" (($output.Trim() -split "`r?`n" | Select-Object -Last 8) -join "`n")
    } else {
      Add-Check $Name "fail" (($output.Trim() -split "`r?`n" | Select-Object -Last 12) -join "`n")
    }
  } catch {
    Add-Check $Name "fail" $_.Exception.Message
  } finally {
    Pop-Location
  }
}

function Test-InstallerArtifacts {
  $msi = Get-ChildItem -Path (Join-Path $root "src-tauri\target\release\bundle\msi") -Filter "*.msi" -ErrorAction SilentlyContinue |
    Sort-Object LastWriteTime -Descending |
    Select-Object -First 1
  $nsis = Get-ChildItem -Path (Join-Path $root "src-tauri\target\release\bundle\nsis") -Filter "*.exe" -ErrorAction SilentlyContinue |
    Sort-Object LastWriteTime -Descending |
    Select-Object -First 1

  if ($msi -and $nsis) {
    Add-Check "installer artifacts" "pass" "MSI: $($msi.FullName)`nNSIS: $($nsis.FullName)"
  } else {
    Add-Check "installer artifacts" "warn" "One or both installer artifacts are missing. Run pnpm tauri build when a release artifact is required."
  }
}

function Test-TauriIdentifier {
  try {
    $configPath = Join-Path $root "src-tauri\tauri.conf.json"
    $config = Get-Content -Encoding utf8 -Path $configPath | ConvertFrom-Json
    $identifier = [string]$config.identifier

    if ($identifier.EndsWith(".app")) {
      Add-Check "tauri identifier" "warn" "Identifier '$identifier' ends with .app. Tauri warns this is not recommended for macOS bundles."
    } else {
      Add-Check "tauri identifier" "pass" "Identifier '$identifier' does not end with .app."
    }
  } catch {
    Add-Check "tauri identifier" "fail" $_.Exception.Message
  }
}

function Test-OllamaAvailability {
  $baseUrl = $env:LV4_OLLAMA_BASE_URL
  if ([string]::IsNullOrWhiteSpace($baseUrl)) {
    $baseUrl = $env:OLLAMA_BASE_URL
  }
  if ([string]::IsNullOrWhiteSpace($baseUrl)) {
    $baseUrl = "http://localhost:11434"
  }
  $baseUrl = $baseUrl.TrimEnd("/")

  $ollama = Get-Command ollama -ErrorAction SilentlyContinue
  if (-not $ollama) {
    Add-Check "ollama command" "warn" "ollama command was not found on PATH. Ollama chat compatibility still needs manual validation on a machine with Ollama installed."
  } else {
    try {
      $version = & ollama --version 2>&1 | Out-String
      Add-Check "ollama command" "pass" $version.Trim()
    } catch {
      Add-Check "ollama command" "warn" "ollama exists but version check failed: $($_.Exception.Message)"
    }
  }

  try {
    $tags = Invoke-RestMethod -Uri "$baseUrl/api/tags" -TimeoutSec 2
    $models = @($tags.models | ForEach-Object { $_.name }) -join ", "
    if ([string]::IsNullOrWhiteSpace($models)) {
      $models = "Ollama service responded but no models were listed."
    }
    Add-Check "ollama service" "pass" "$baseUrl -> $models"
  } catch {
    Add-Check "ollama service" "warn" "$baseUrl/api/tags did not respond: $($_.Exception.Message)"
  }
}

function Test-AgentStdioSmoke {
  Write-Host "== speakmate agent stdio"
  Push-Location (Join-Path $root "src-tauri")
  try {
    $global:LASTEXITCODE = 0
    $inputLines = @(
      '{"id":"init","method":"initialize"}'
      '{"id":"tools","method":"tools/list"}'
      '{"id":"characters","tool":"list_characters","arguments":{"limit":5}}'
      '{"id":"status","tool":"get_app_status","arguments":{}}'
      '{"id":"stop","method":"shutdown"}'
    )
    $output = $inputLines | cargo run --quiet --bin voxverse-cli -- agent serve --stdio --read-only 2>&1 | Out-String
    $exitCode = $LASTEXITCODE
    $escape = [char]27
    $output = $output -replace "$escape\[[0-9;]*m", ""

    if ($exitCode -eq 0 -and $output -match '"id":"characters"' -and $output -match '"characters"' -and $output -match '"id":"stop"' -and $output -match '"shutdown":true') {
      Add-Check "speakmate agent stdio" "pass" (($output.Trim() -split "`r?`n" | Select-Object -Last 8) -join "`n")
    } else {
      Add-Check "speakmate agent stdio" "fail" (($output.Trim() -split "`r?`n" | Select-Object -Last 12) -join "`n")
    }
  } catch {
    Add-Check "speakmate agent stdio" "fail" $_.Exception.Message
  } finally {
    Pop-Location
  }
}

function Test-McpStdioSmoke {
  Write-Host "== speakmate mcp stdio"
  Push-Location (Join-Path $root "src-tauri")
  try {
    $global:LASTEXITCODE = 0
    $inputLines = @(
      '{"jsonrpc":"2.0","id":"init","method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"lv4-smoke","version":"0.0.0"}}}'
      '{"jsonrpc":"2.0","method":"notifications/initialized"}'
      '{"jsonrpc":"2.0","id":"tools","method":"tools/list"}'
      '{"jsonrpc":"2.0","id":"characters","method":"tools/call","params":{"name":"list_characters","arguments":{"limit":5}}}'
      '{"jsonrpc":"2.0","id":"status","method":"tools/call","params":{"name":"get_app_status","arguments":{}}}'
      '{"jsonrpc":"2.0","id":"stop","method":"shutdown"}'
    )
    $output = $inputLines | cargo run --quiet --bin voxverse-cli -- agent serve --stdio --read-only 2>&1 | Out-String
    $exitCode = $LASTEXITCODE
    $escape = [char]27
    $output = $output -replace "$escape\[[0-9;]*m", ""

    if (
      $exitCode -eq 0 `
      -and $output -match '"jsonrpc":"2\.0"' `
      -and $output -match '"serverInfo"' `
      -and $output -match '"tools"' `
      -and $output -match '"characters"' `
      -and $output -match '"structuredContent"' `
      -and $output -match '"shutdown":true'
    ) {
      Add-Check "speakmate mcp stdio" "pass" (($output.Trim() -split "`r?`n" | Select-Object -Last 8) -join "`n")
    } else {
      Add-Check "speakmate mcp stdio" "fail" (($output.Trim() -split "`r?`n" | Select-Object -Last 12) -join "`n")
    }
  } catch {
    Add-Check "speakmate mcp stdio" "fail" $_.Exception.Message
  } finally {
    Pop-Location
  }
}

function Test-AgentWriteSmoke {
  Write-Host "== speakmate agent write guard"
  $sourceDb = Get-SpeakMateDbCandidate
  if ([string]::IsNullOrWhiteSpace($sourceDb)) {
    Add-Check "speakmate agent write guard" "warn" "No local SpeakMate database was found, so the agent write smoke test was skipped."
    return
  }

  $tmpRoot = Join-Path ([System.IO.Path]::GetTempPath()) "speakmate-agent-write-smoke"
  if (-not (Test-Path $tmpRoot)) {
    New-Item -ItemType Directory -Path $tmpRoot | Out-Null
  }

  $stamp = Get-Date -Format "yyyyMMdd-HHmmss"
  $tmpDb = Join-Path $tmpRoot "agent-write-$stamp.db"
  $auditLog = Join-Path $tmpRoot "speakmate-audit.jsonl"

  try {
    Copy-Item -LiteralPath $sourceDb -Destination $tmpDb -Force
    Push-Location (Join-Path $root "src-tauri")
    try {
      $characterId = $null
      $global:LASTEXITCODE = 0
      $characterOutput = & cargo run --quiet --bin voxverse-cli -- --db "$tmpDb" character list --json 2>&1 | Out-String
      if ($LASTEXITCODE -eq 0) {
        try {
          $characterReport = $characterOutput | ConvertFrom-Json
          $characters = @($characterReport.characters)
          if ($characters.Count -gt 0) {
            $characterId = [string]$characters[0].id
          }
        } catch {
          $characterId = $null
        }
      }

      $sessionId = $null
      if (-not [string]::IsNullOrWhiteSpace($characterId)) {
        $global:LASTEXITCODE = 0
        $sessionOutput = & cargo run --quiet --bin voxverse-cli -- --db "$tmpDb" session start --character "$characterId" --title "Lv4 append smoke" --json --yes 2>&1 | Out-String
        if ($LASTEXITCODE -eq 0) {
          try {
            $sessionReport = $sessionOutput | ConvertFrom-Json
            $sessionId = [string]$sessionReport.session.id
          } catch {
            $sessionId = $null
          }
        }
      }

      $global:LASTEXITCODE = 0
      $readOnlyLines = @()
      if (-not [string]::IsNullOrWhiteSpace($characterId)) {
        $readOnlyLines += (@{
          jsonrpc = "2.0"
          id = "start"
          method = "tools/call"
          params = @{
            name = "start_practice_session"
            arguments = @{
              character_id = $characterId
              title = "Lv4 smoke"
            }
          }
        } | ConvertTo-Json -Compress -Depth 8)
      }
      if (-not [string]::IsNullOrWhiteSpace($sessionId)) {
        $readOnlyLines += (@{
          jsonrpc = "2.0"
          id = "append"
          method = "tools/call"
          params = @{
            name = "append_session_message"
            arguments = @{
              session_id = $sessionId
              role = "user"
              content = "Hello from the Lv4 smoke test."
            }
          }
        } | ConvertTo-Json -Compress -Depth 8)
      }
      $readOnlyLines += @(
        '{"jsonrpc":"2.0","id":"switch","method":"tools/call","params":{"name":"switch_provider_profile","arguments":{"profile_id":"default"}}}'
        '{"jsonrpc":"2.0","id":"stop","method":"shutdown"}'
      )
      $readOnlyOutput = $readOnlyLines | cargo run --quiet --bin voxverse-cli -- --db "$tmpDb" agent serve --stdio --read-only 2>&1 | Out-String
      $readOnlyExit = $LASTEXITCODE

      $global:LASTEXITCODE = 0
      $writeLines = @(
        '{"jsonrpc":"2.0","id":"tools","method":"tools/list"}'
      )
      if (-not [string]::IsNullOrWhiteSpace($characterId)) {
        $writeLines += (@{
          jsonrpc = "2.0"
          id = "start"
          method = "tools/call"
          params = @{
            name = "start_practice_session"
            arguments = @{
              character_id = $characterId
              title = "Lv4 smoke"
            }
          }
        } | ConvertTo-Json -Compress -Depth 8)
      }
      if (-not [string]::IsNullOrWhiteSpace($sessionId)) {
        $writeLines += (@{
          jsonrpc = "2.0"
          id = "append"
          method = "tools/call"
          params = @{
            name = "append_session_message"
            arguments = @{
              session_id = $sessionId
              role = "assistant"
              content = "Hello, I am ready to practice."
            }
          }
        } | ConvertTo-Json -Compress -Depth 8)
      }
      $writeLines += @(
        '{"jsonrpc":"2.0","id":"switch","method":"tools/call","params":{"name":"switch_provider_profile","arguments":{"profile_id":"default"}}}'
        '{"jsonrpc":"2.0","id":"audit","method":"tools/call","params":{"name":"list_audit_events","arguments":{"limit":5}}}'
        '{"jsonrpc":"2.0","id":"stop","method":"shutdown"}'
      )
      $writeOutput = $writeLines | cargo run --quiet --bin voxverse-cli -- --db "$tmpDb" agent serve --stdio --allow-writes --yes 2>&1 | Out-String
      $writeExit = $LASTEXITCODE

      $global:LASTEXITCODE = 0
      $filteredLines = @(
        '{"id":"tools","method":"tools/list"}'
        '{"id":"switch","tool":"switch_provider_profile","arguments":{"profile_id":"default"}}'
        '{"id":"stop","method":"shutdown"}'
      )
      $filteredOutput = $filteredLines | cargo run --quiet --bin voxverse-cli -- --db "$tmpDb" agent serve --stdio --allow-writes --yes --allow-tool append_session_message 2>&1 | Out-String
      $filteredExit = $LASTEXITCODE

      $global:LASTEXITCODE = 0
      $networkBlockedLines = @(
        '{"id":"send","tool":"send_message","arguments":{"session_id":"blocked-smoke","content":"This must not call the network."}}'
        '{"id":"stop","method":"shutdown"}'
      )
      $networkBlockedOutput = $networkBlockedLines | cargo run --quiet --bin voxverse-cli -- --db "$tmpDb" agent serve --stdio --allow-writes --yes --allow-tool send_message 2>&1 | Out-String
      $networkBlockedExit = $LASTEXITCODE

      $global:LASTEXITCODE = 0
      $networkToolsOutput = & cargo run --quiet --bin voxverse-cli -- --db "$tmpDb" agent tools list --json --allow-writes --allow-network --allow-tool send_message 2>&1 | Out-String
      $networkToolsExit = $LASTEXITCODE

      $escape = [char]27
      $readOnlyOutput = $readOnlyOutput -replace "$escape\[[0-9;]*m", ""
      $writeOutput = $writeOutput -replace "$escape\[[0-9;]*m", ""
      $filteredOutput = $filteredOutput -replace "$escape\[[0-9;]*m", ""
      $networkBlockedOutput = $networkBlockedOutput -replace "$escape\[[0-9;]*m", ""
      $networkToolsOutput = $networkToolsOutput -replace "$escape\[[0-9;]*m", ""
      $combinedOutput = "$readOnlyOutput`n$writeOutput`n$filteredOutput`n$networkBlockedOutput`n$networkToolsOutput"
      $sessionStartOk = $true
      if (-not [string]::IsNullOrWhiteSpace($characterId)) {
        $sessionStartOk = $readOnlyOutput -match 'start_practice_session requires agent serve --allow-writes --yes' `
          -and $writeOutput -match '"start_practice_session"' `
          -and $writeOutput -match 'practice\.session\.start' `
          -and $writeOutput -match '"target_session_id"'
      }
      $messageAppendOk = $true
      if (-not [string]::IsNullOrWhiteSpace($sessionId)) {
        $messageAppendOk = $readOnlyOutput -match 'append_session_message requires agent serve --allow-writes --yes' `
          -and $writeOutput -match '"append_session_message"' `
          -and $writeOutput -match 'practice\.message\.append' `
          -and $writeOutput -match '"target_message_id"'
      }
      $allowlistOk = $filteredExit -eq 0 `
        -and $filteredOutput -match '"write_allowlist":\["append_session_message"\]' `
        -and $filteredOutput -match '"append_session_message"' `
        -and $filteredOutput -match 'switch_provider_profile is not enabled by this agent server allowlist'
      $networkBoundaryOk = $networkBlockedExit -eq 0 `
        -and $networkBlockedOutput -match 'send_message requires agent serve --allow-network' `
        -and $networkToolsExit -eq 0 `
        -and $networkToolsOutput -match '"network_enabled"\s*:\s*true' `
        -and $networkToolsOutput -match '"write_allowlist"\s*:\s*\[' `
        -and $networkToolsOutput -match '"send_message"'

      if (
        $readOnlyExit -eq 0 `
        -and $writeExit -eq 0 `
        -and $allowlistOk `
        -and $networkBoundaryOk `
        -and $readOnlyOutput -match 'requires agent serve --allow-writes --yes' `
        -and $sessionStartOk `
        -and $messageAppendOk `
        -and $writeOutput -match '"switch_provider_profile"' `
        -and $writeOutput -match '"actor":"speakmate-agent"' `
        -and $writeOutput -match '"shutdown":true'
      ) {
        Add-Check "speakmate agent write guard" "pass" (($combinedOutput.Trim() -split "`r?`n" | Select-Object -Last 8) -join "`n")
      } else {
        Add-Check "speakmate agent write guard" "fail" (($combinedOutput.Trim() -split "`r?`n" | Select-Object -Last 12) -join "`n")
      }
    } finally {
      Pop-Location
    }
  } catch {
    Add-Check "speakmate agent write guard" "fail" $_.Exception.Message
  } finally {
    if (Test-Path $tmpDb) {
      Remove-Item -LiteralPath $tmpDb -Force
    }
    if (Test-Path $auditLog) {
      Remove-Item -LiteralPath $auditLog -Force
    }
  }
}

function Get-FreeTcpPort {
  $listener = [System.Net.Sockets.TcpListener]::new([System.Net.IPAddress]::Loopback, 0)
  try {
    $listener.Start()
    return ([System.Net.IPEndPoint]$listener.LocalEndpoint).Port
  } finally {
    $listener.Stop()
  }
}

function Start-MockChatCompletionServer {
  param(
    [int]$Port,
    [int]$RequestCount = 1
  )

  Start-Job -ArgumentList $Port, $RequestCount -ScriptBlock {
    param([int]$Port, [int]$RequestCount)

    $listener = [System.Net.Sockets.TcpListener]::new([System.Net.IPAddress]::Parse("127.0.0.1"), $Port)
    try {
      $listener.Start()
      for ($index = 0; $index -lt $RequestCount; $index++) {
        $client = $null
        try {
          $client = $listener.AcceptTcpClient()
          $stream = $client.GetStream()
          $buffer = New-Object byte[] 4096
          $requestText = New-Object System.Text.StringBuilder
          $deadline = (Get-Date).AddSeconds(10)

          while ((Get-Date) -lt $deadline) {
            if ($stream.DataAvailable) {
              $read = $stream.Read($buffer, 0, $buffer.Length)
              if ($read -le 0) {
                break
              }
              [void]$requestText.Append([System.Text.Encoding]::UTF8.GetString($buffer, 0, $read))
              if ($requestText.ToString().Contains("`r`n`r`n")) {
                break
              }
            } else {
              Start-Sleep -Milliseconds 20
            }
          }

          $body = '{"choices":[{"message":{"content":"Mock assistant response from Lv4 smoke."}}]}'
          $bodyBytes = [System.Text.Encoding]::UTF8.GetBytes($body)
          $headers = "HTTP/1.1 200 OK`r`nContent-Type: application/json`r`nContent-Length: $($bodyBytes.Length)`r`nConnection: close`r`n`r`n"
          $headerBytes = [System.Text.Encoding]::ASCII.GetBytes($headers)
          $stream.Write($headerBytes, 0, $headerBytes.Length)
          $stream.Write($bodyBytes, 0, $bodyBytes.Length)
          $stream.Flush()
        } finally {
          if ($client) {
            $client.Close()
          }
        }
      }
    } finally {
      $listener.Stop()
    }
  }
}

function Test-SendMessageMockSmoke {
  Write-Host "== speakmate send-message mock"
  $sourceDb = Get-SpeakMateDbCandidate
  if ([string]::IsNullOrWhiteSpace($sourceDb)) {
    Add-Check "speakmate send-message mock" "warn" "No local SpeakMate database was found, so the send-message mock smoke test was skipped."
    return
  }

  $tmpRoot = Join-Path ([System.IO.Path]::GetTempPath()) "speakmate-send-message-smoke"
  if (-not (Test-Path $tmpRoot)) {
    New-Item -ItemType Directory -Path $tmpRoot | Out-Null
  }

  $stamp = Get-Date -Format "yyyyMMdd-HHmmss"
  $tmpDb = Join-Path $tmpRoot "send-message-$stamp.db"
  $auditLog = Join-Path $tmpRoot "speakmate-audit.jsonl"
  $mockJob = $null
  $oldBaseUrl = $env:SPEAKMATE_CLI_SEND_BASE_URL
  $oldProvider = $env:SPEAKMATE_CLI_SEND_PROVIDER
  $oldModel = $env:SPEAKMATE_CLI_SEND_MODEL
  $oldApiKey = $env:SPEAKMATE_CLI_SEND_API_KEY

  try {
    Copy-Item -LiteralPath $sourceDb -Destination $tmpDb -Force
    Push-Location (Join-Path $root "src-tauri")
    try {
      $global:LASTEXITCODE = 0
      $characterOutput = & cargo run --quiet --bin voxverse-cli -- --db "$tmpDb" character list --json 2>&1 | Out-String
      if ($LASTEXITCODE -ne 0) {
        Add-Check "speakmate send-message mock" "fail" (($characterOutput.Trim() -split "`r?`n" | Select-Object -Last 8) -join "`n")
        return
      }

      $characterReport = $characterOutput | ConvertFrom-Json
      $characters = @($characterReport.characters)
      if ($characters.Count -eq 0) {
        Add-Check "speakmate send-message mock" "warn" "No local characters were found, so the send-message mock smoke test was skipped."
        return
      }
      $characterId = [string]$characters[0].id

      $sessionOutput = & cargo run --quiet --bin voxverse-cli -- --db "$tmpDb" session start --character "$characterId" --title "Lv4 send-message smoke" --json --yes 2>&1 | Out-String
      if ($LASTEXITCODE -ne 0) {
        Add-Check "speakmate send-message mock" "fail" (($sessionOutput.Trim() -split "`r?`n" | Select-Object -Last 8) -join "`n")
        return
      }
      $sessionReport = $sessionOutput | ConvertFrom-Json
      $sessionId = [string]$sessionReport.session.id

      $port = Get-FreeTcpPort
      $mockJob = Start-MockChatCompletionServer -Port $port -RequestCount 5
      Start-Sleep -Milliseconds 600

      $env:SPEAKMATE_CLI_SEND_BASE_URL = "http://127.0.0.1:$port/v1"
      $env:SPEAKMATE_CLI_SEND_PROVIDER = "custom"
      $env:SPEAKMATE_CLI_SEND_MODEL = "lv4-mock-model"
      $env:SPEAKMATE_CLI_SEND_API_KEY = ""

      $global:LASTEXITCODE = 0
      $sendOutput = & cargo run --quiet --bin voxverse-cli -- --db "$tmpDb" session send-message --session "$sessionId" --content "Hello from the Lv4 mock send-message smoke." --json --yes --allow-network 2>&1 | Out-String
      $sendExit = $LASTEXITCODE

      $appendRetryOutput = & cargo run --quiet --bin voxverse-cli -- --db "$tmpDb" session append-message --session "$sessionId" --role user --content "Please retry the last user message from the Lv4 smoke." --json --yes 2>&1 | Out-String
      $appendRetryExit = $LASTEXITCODE
      $retryOutput = & cargo run --quiet --bin voxverse-cli -- --db "$tmpDb" session retry-last --session "$sessionId" --json --yes --allow-network 2>&1 | Out-String
      $retryExit = $LASTEXITCODE

      $appendAgentRetryOutput = & cargo run --quiet --bin voxverse-cli -- --db "$tmpDb" session append-message --session "$sessionId" --role user --content "Please retry the last user message through the Lv4 agent smoke." --json --yes 2>&1 | Out-String
      $appendAgentRetryExit = $LASTEXITCODE
      $agentRetryLines = @(
        '{"id":"init","method":"initialize"}'
        ('{"id":"retry","tool":"retry_last_message","arguments":{"session_id":"' + $sessionId + '"}}')
        '{"id":"stop","method":"shutdown"}'
      )
      $agentRetryOutput = $agentRetryLines | cargo run --quiet --bin voxverse-cli -- --db "$tmpDb" agent serve --stdio --allow-writes --yes --allow-network --allow-tool retry_last_message 2>&1 | Out-String
      $agentRetryExit = $LASTEXITCODE

      $coachOutput = & cargo run --quiet --bin voxverse-cli -- --db "$tmpDb" session coach-report --session "$sessionId" --json --allow-network 2>&1 | Out-String
      $coachExit = $LASTEXITCODE
      $agentCoachLines = @(
        '{"id":"init","method":"initialize"}'
        ('{"id":"coach","tool":"generate_session_coaching_report","arguments":{"session_id":"' + $sessionId + '"}}')
        '{"id":"stop","method":"shutdown"}'
      )
      $agentCoachOutput = $agentCoachLines | cargo run --quiet --bin voxverse-cli -- --db "$tmpDb" agent serve --stdio --read-only --allow-network 2>&1 | Out-String
      $agentCoachExit = $LASTEXITCODE

      $exportOutput = & cargo run --quiet --bin voxverse-cli -- --db "$tmpDb" session export --session "$sessionId" --json 2>&1 | Out-String
      $exportExit = $LASTEXITCODE
      $auditOutput = & cargo run --quiet --bin voxverse-cli -- --db "$tmpDb" audit list --json --limit 8 2>&1 | Out-String
      $auditExit = $LASTEXITCODE
      $filteredAuditOutput = & cargo run --quiet --bin voxverse-cli -- --db "$tmpDb" audit list --json --limit 8 --operation practice.message.send --actor speakmate-cli --result success --session "$sessionId" 2>&1 | Out-String
      $filteredAuditExit = $LASTEXITCODE
      $retryAuditOutput = & cargo run --quiet --bin voxverse-cli -- --db "$tmpDb" audit list --json --limit 8 --operation practice.message.retry_last --actor speakmate-cli --result success --session "$sessionId" 2>&1 | Out-String
      $retryAuditExit = $LASTEXITCODE
      $agentRetryAuditOutput = & cargo run --quiet --bin voxverse-cli -- --db "$tmpDb" audit list --json --limit 8 --operation practice.message.retry_last --actor speakmate-agent --result success --session "$sessionId" 2>&1 | Out-String
      $agentRetryAuditExit = $LASTEXITCODE

      $escape = [char]27
      $sendOutput = $sendOutput -replace "$escape\[[0-9;]*m", ""
      $appendRetryOutput = $appendRetryOutput -replace "$escape\[[0-9;]*m", ""
      $retryOutput = $retryOutput -replace "$escape\[[0-9;]*m", ""
      $appendAgentRetryOutput = $appendAgentRetryOutput -replace "$escape\[[0-9;]*m", ""
      $agentRetryOutput = $agentRetryOutput -replace "$escape\[[0-9;]*m", ""
      $coachOutput = $coachOutput -replace "$escape\[[0-9;]*m", ""
      $agentCoachOutput = $agentCoachOutput -replace "$escape\[[0-9;]*m", ""
      $exportOutput = $exportOutput -replace "$escape\[[0-9;]*m", ""
      $auditOutput = $auditOutput -replace "$escape\[[0-9;]*m", ""
      $filteredAuditOutput = $filteredAuditOutput -replace "$escape\[[0-9;]*m", ""
      $retryAuditOutput = $retryAuditOutput -replace "$escape\[[0-9;]*m", ""
      $agentRetryAuditOutput = $agentRetryAuditOutput -replace "$escape\[[0-9;]*m", ""
      $combinedOutput = "$sendOutput`n$appendRetryOutput`n$retryOutput`n$appendAgentRetryOutput`n$agentRetryOutput`n$coachOutput`n$agentCoachOutput`n$exportOutput`n$auditOutput`n$filteredAuditOutput`n$retryAuditOutput`n$agentRetryAuditOutput"
      $targetSessionPattern = '"target_session_id"\s*:\s*"' + [regex]::Escape($sessionId) + '"'

      if (
        $sendExit -eq 0 `
        -and $appendRetryExit -eq 0 `
        -and $retryExit -eq 0 `
        -and $appendAgentRetryExit -eq 0 `
        -and $agentRetryExit -eq 0 `
        -and $coachExit -eq 0 `
        -and $agentCoachExit -eq 0 `
        -and $exportExit -eq 0 `
        -and $auditExit -eq 0 `
        -and $filteredAuditExit -eq 0 `
        -and $retryAuditExit -eq 0 `
        -and $agentRetryAuditExit -eq 0 `
        -and $sendOutput -match 'Mock assistant response from Lv4 smoke' `
        -and $sendOutput -match '"assistant_message"' `
        -and $retryOutput -match 'Mock assistant response from Lv4 smoke' `
        -and $retryOutput -match '"assistant_message"' `
        -and $agentRetryOutput -match 'Mock assistant response from Lv4 smoke' `
        -and $agentRetryOutput -match '"assistant_message"' `
        -and $coachOutput -match 'Mock assistant response from Lv4 smoke' `
        -and $coachOutput -match '"coaching_markdown"' `
        -and $agentCoachOutput -match 'Mock assistant response from Lv4 smoke' `
        -and $agentCoachOutput -match '"coaching_markdown"' `
        -and $exportOutput -match 'Mock assistant response from Lv4 smoke' `
        -and $exportOutput -match '"summary"' `
        -and $exportOutput -match '"transcript_markdown"' `
        -and $exportOutput -match 'VoxVerse Session Report' `
        -and $auditOutput -match 'practice\.message\.send' `
        -and $auditOutput -match 'practice\.message\.retry_last' `
        -and $auditOutput -match '"result"\s*:\s*"success"' `
        -and $filteredAuditOutput -match '"filters"' `
        -and $filteredAuditOutput -match '"actor"\s*:\s*"speakmate-cli"' `
        -and $retryAuditOutput -match '"operation"\s*:\s*"practice.message.retry_last"' `
        -and $agentRetryAuditOutput -match '"actor"\s*:\s*"speakmate-agent"' `
        -and $filteredAuditOutput -match $targetSessionPattern
      ) {
        Add-Check "speakmate send-message mock" "pass" (($combinedOutput.Trim() -split "`r?`n" | Select-Object -Last 10) -join "`n")
      } else {
        Add-Check "speakmate send-message mock" "fail" (($combinedOutput.Trim() -split "`r?`n" | Select-Object -Last 14) -join "`n")
      }
    } finally {
      Pop-Location
    }
  } catch {
    Add-Check "speakmate send-message mock" "fail" $_.Exception.Message
  } finally {
    $env:SPEAKMATE_CLI_SEND_BASE_URL = $oldBaseUrl
    $env:SPEAKMATE_CLI_SEND_PROVIDER = $oldProvider
    $env:SPEAKMATE_CLI_SEND_MODEL = $oldModel
    $env:SPEAKMATE_CLI_SEND_API_KEY = $oldApiKey

    if ($mockJob) {
      Wait-Job $mockJob -Timeout 2 | Out-Null
      if ($mockJob.State -eq "Running") {
        Stop-Job $mockJob -Force
      }
      Remove-Job $mockJob -Force
    }
    if (Test-Path $tmpDb) {
      Remove-Item -LiteralPath $tmpDb -Force
    }
    if (Test-Path $auditLog) {
      Remove-Item -LiteralPath $auditLog -Force
    }
  }
}

function Get-SpeakMateDbCandidate {
  $candidates = @()
  if (-not [string]::IsNullOrWhiteSpace($env:SPEAKMATE_DB)) {
    $candidates += $env:SPEAKMATE_DB
  }
  if (-not [string]::IsNullOrWhiteSpace($env:LOCALAPPDATA)) {
    $candidates += (Join-Path $env:LOCALAPPDATA "com.voxverse.desktop\voxverse.db")
    $candidates += (Join-Path $env:LOCALAPPDATA "com.voxverse.app\voxverse.db")
    $candidates += (Join-Path $env:LOCALAPPDATA "com.ai-speaking.desktop\voxverse.db")
    $candidates += (Join-Path $env:LOCALAPPDATA "com.ai-speaking.desktop\speakmate.db")
    $candidates += (Join-Path $env:LOCALAPPDATA "com.ai-speaking.app\speakmate.db")
    $candidates += (Join-Path $env:LOCALAPPDATA "SpeakMate\speakmate.db")
  }

  foreach ($candidate in $candidates) {
    if (Test-Path $candidate) {
      return (Resolve-Path $candidate).Path
    }
  }

  return $null
}

function Test-ProfileSwitchSmoke {
  Write-Host "== speakmate profile switch"
  $sourceDb = Get-SpeakMateDbCandidate
  if ([string]::IsNullOrWhiteSpace($sourceDb)) {
    Add-Check "speakmate profile switch" "warn" "No local SpeakMate database was found, so the write-operation smoke test was skipped."
    return
  }

  $tmpRoot = Join-Path ([System.IO.Path]::GetTempPath()) "speakmate-cli-smoke"
  if (-not (Test-Path $tmpRoot)) {
    New-Item -ItemType Directory -Path $tmpRoot | Out-Null
  }

  $stamp = Get-Date -Format "yyyyMMdd-HHmmss"
  $tmpDb = Join-Path $tmpRoot "switch-smoke-$stamp.db"
  $auditLog = Join-Path $tmpRoot "speakmate-audit.jsonl"

  try {
    Copy-Item -LiteralPath $sourceDb -Destination $tmpDb -Force

    Push-Location (Join-Path $root "src-tauri")
    try {
      $global:LASTEXITCODE = 0
      $output = & cargo run --quiet --bin voxverse-cli -- --db "$tmpDb" config profiles switch default --json --yes 2>&1 | Out-String
      $exitCode = $LASTEXITCODE
      $escape = [char]27
      $output = $output -replace "$escape\[[0-9;]*m", ""

      $auditOutput = & cargo run --quiet --bin voxverse-cli -- --db "$tmpDb" audit list --json --limit 5 2>&1 | Out-String
      $auditExitCode = $LASTEXITCODE
      $auditOutput = $auditOutput -replace "$escape\[[0-9;]*m", ""
      $combinedOutput = "$output`n$auditOutput"

      if (
        $exitCode -eq 0 `
        -and $auditExitCode -eq 0 `
        -and $output -match '"active_profile"' `
        -and $output -match '"audit_log"' `
        -and $auditOutput -match 'config\.profiles\.switch'
      ) {
        Add-Check "speakmate profile switch" "pass" (($combinedOutput.Trim() -split "`r?`n" | Select-Object -Last 8) -join "`n")
      } else {
        Add-Check "speakmate profile switch" "fail" (($combinedOutput.Trim() -split "`r?`n" | Select-Object -Last 12) -join "`n")
      }
    } finally {
      Pop-Location
    }
  } catch {
    Add-Check "speakmate profile switch" "fail" $_.Exception.Message
  } finally {
    if (Test-Path $tmpDb) {
      Remove-Item -LiteralPath $tmpDb -Force
    }
    if (Test-Path $auditLog) {
      Remove-Item -LiteralPath $auditLog -Force
    }
  }
}

function Write-Report {
  if ([string]::IsNullOrWhiteSpace($ReportPath)) {
    $stamp = Get-Date -Format "yyyyMMdd-HHmmss"
    $ReportPath = Join-Path $root "validation-reports\lv4-smoke-report-$stamp.md"
  }

  $generated = Get-Date -Format "yyyy-MM-ddTHH:mm:sszzz"
  $lines = New-Object System.Collections.Generic.List[string]
  $lines.Add("# Lv.4 Automated Smoke Report") | Out-Null
  $lines.Add("") | Out-Null
  $lines.Add("> Generated: $generated") | Out-Null
  $lines.Add("> Machine: $env:COMPUTERNAME") | Out-Null
  $lines.Add("> Full build: $($FullBuild.IsPresent)") | Out-Null
  $lines.Add("") | Out-Null
  $lines.Add("| Check | Status | Details |") | Out-Null
  $lines.Add("| --- | --- | --- |") | Out-Null

  foreach ($check in $checks) {
    $details = (($check.Details -replace "`r?`n", "<br>") -replace "\|", "\|")
    $lines.Add("| $($check.Name) | $($check.Status) | $details |") | Out-Null
  }

  $lines.Add("") | Out-Null
  $lines.Add("## Notes") | Out-Null
  $lines.Add("") | Out-Null
  $lines.Add("- This report does not read or print API keys.") | Out-Null
  $lines.Add("- Set `LV4_OLLAMA_BASE_URL` or `OLLAMA_BASE_URL` to validate a non-default Ollama endpoint.") | Out-Null
  $lines.Add("- OpenAI, Whisper, and browser STT still require an interactive app session for complete validation.") | Out-Null
  $lines.Add("- Use `lv4-validation-checklist.md` for the remaining manual checks.") | Out-Null

  $resolvedReportPath = if ([System.IO.Path]::IsPathRooted($ReportPath)) {
    $ReportPath
  } else {
    Join-Path $root $ReportPath
  }

  $reportDir = Split-Path -Parent $resolvedReportPath
  if (-not (Test-Path $reportDir)) {
    New-Item -ItemType Directory -Path $reportDir | Out-Null
  }

  Set-Content -Encoding utf8 -Path $resolvedReportPath -Value $lines
  Write-Host "Report written to $resolvedReportPath"
}

Invoke-Check "pnpm build" "pnpm build"
Invoke-Check "cargo check" "cargo check" (Join-Path $root "src-tauri")
Invoke-Check "speakmate cli diagnostics" "cargo run --quiet --bin voxverse-cli -- diagnostics run --json" (Join-Path $root "src-tauri")
Invoke-Check "speakmate cli characters" "cargo run --quiet --bin voxverse-cli -- character list --json" (Join-Path $root "src-tauri")
Invoke-Check "speakmate cli sessions" "cargo run --quiet --bin voxverse-cli -- session list --json" (Join-Path $root "src-tauri")
Invoke-Check "speakmate agent tools" "cargo run --quiet --bin voxverse-cli -- agent tools list --json" (Join-Path $root "src-tauri")
Test-AgentStdioSmoke
Test-McpStdioSmoke
Test-ProfileSwitchSmoke
Test-AgentWriteSmoke
Test-SendMessageMockSmoke

if ($FullBuild) {
  Invoke-Check "pnpm tauri build" "pnpm tauri build"
} else {
  Add-Check "pnpm tauri build" "skip" "Skipped by default. Run pnpm validate:lv4:full to include desktop bundling."
}

Test-InstallerArtifacts
Test-TauriIdentifier
Test-OllamaAvailability
Write-Report

$failed = @($checks | Where-Object { $_.Status -eq "fail" })
if ($failed.Count -gt 0) {
  exit 1
}

exit 0
