param(
    [string]$Binary = (Join-Path $PSScriptRoot "..\target\release\aether-lab.exe"),
    [string]$Output = (Join-Path $PSScriptRoot "..\research\results\dae-native-performance.json")
)
$ErrorActionPreference = "Stop"
$resolvedBinary = (Resolve-Path -LiteralPath $Binary).Path
$resolvedOutput = [IO.Path]::GetFullPath($Output)
$directory = Split-Path -Parent $resolvedOutput
New-Item -ItemType Directory -Force -Path $directory | Out-Null
$stem = Join-Path ([IO.Path]::GetTempPath()) ("aether-perf-" + [guid]::NewGuid().ToString("N"))
$stdoutPath = "$stem.stdout"
$stderrPath = "$stem.stderr"
$processor = Get-CimInstance Win32_Processor | Select-Object -First 1
$os = Get-CimInstance Win32_OperatingSystem
$logical = [Environment]::ProcessorCount
$started = [DateTime]::UtcNow
$childProcess = Start-Process -FilePath $resolvedBinary -ArgumentList "perf-lab" -PassThru -RedirectStandardOutput $stdoutPath -RedirectStandardError $stderrPath
$cpuStart = 0.0
$cpuTotal = 0.0
$peakWorkingSet = 0L
$peakPrivate = 0L
try {
    $sampleCount = 0
    while (-not $childProcess.HasExited) {
        $current = Get-Process -Id $childProcess.Id -ErrorAction SilentlyContinue
        if ($null -ne $current) {
            if ($sampleCount -eq 0) { $cpuStart = $current.CPU }
            $cpuTotal = $current.CPU
            $peakWorkingSet = [Math]::Max($peakWorkingSet, $current.PeakWorkingSet64)
            $peakPrivate = [Math]::Max($peakPrivate, $current.PeakPagedMemorySize64)
            $sampleCount++
        }
        Start-Sleep -Milliseconds 20
        $childProcess.Refresh()
    }
    $childProcess.WaitForExit()
    if ((Get-Item -LiteralPath $stdoutPath).Length -eq 0 -or (Get-Item -LiteralPath $stderrPath).Length -gt 0) {
        throw "perf-lab produced no clean JSON output: $(Get-Content -Raw $stderrPath)"
    }
    $elapsed = ([DateTime]::UtcNow - $started).TotalSeconds
    $cpuSeconds = [Math]::Max(0, $cpuTotal - $cpuStart)
    $report = Get-Content -Raw $stdoutPath | ConvertFrom-Json
    $hostMeasurement = @{
        processor = $processor.Name.Trim()
        logical_processors = $logical
        installed_memory_bytes = [int64]$os.TotalVisibleMemorySize * 1024
        os_caption = $os.Caption
        os_version = $os.Version
        elapsed_seconds_including_process_startup = $elapsed
        parent_process_cpu_seconds = $cpuSeconds
        cpu_percent_of_one_logical_processor = if ($null -ne $cpuSeconds -and $elapsed -gt 0) { 100 * $cpuSeconds / $elapsed } else { $null }
        cpu_percent_of_machine_capacity = if ($null -ne $cpuSeconds -and $elapsed -gt 0) { 100 * $cpuSeconds / ($elapsed * $logical) } else { $null }
        peak_working_set_bytes_sampled = $peakWorkingSet
        peak_private_bytes_sampled = $peakPrivate
        process_sampling_interval_ms = 20
        process_sampling_points = $sampleCount
        cpu_scope = "benchmark controller process only; child API and reviewed workers are separate processes"
    }
    $report | Add-Member -NotePropertyName host_measurement -NotePropertyValue $hostMeasurement -Force
    $report.build | Add-Member -NotePropertyName rustc_verbose -NotePropertyValue ((& rustc -Vv | Out-String).Trim()) -Force
    $json = $report | ConvertTo-Json -Depth 40
    [IO.File]::WriteAllText($resolvedOutput, $json, (New-Object System.Text.UTF8Encoding($false)))
    Write-Output "Saved native performance samples to $resolvedOutput"
    Write-Output "Parent CPU $cpuSeconds s; sampled peak working set $peakWorkingSet bytes; elapsed $elapsed s"
} finally {
    Remove-Item -LiteralPath $stdoutPath,$stderrPath -Force -ErrorAction SilentlyContinue
}
