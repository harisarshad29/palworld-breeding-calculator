# Run cargo without PowerShell treating stderr as a terminating error.
param(
    [Parameter(Mandatory = $true, ValueFromRemainingArguments = $true)]
    [string[]]$CargoArgs
)

$prev = $ErrorActionPreference
$ErrorActionPreference = "Continue"
& cargo @CargoArgs 2>&1 | ForEach-Object { Write-Host $_ }
$code = $LASTEXITCODE
$ErrorActionPreference = $prev
exit $code
