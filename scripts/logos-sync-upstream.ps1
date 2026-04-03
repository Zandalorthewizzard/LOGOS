param(
    [string]$BaseBranch = "main",
    [string]$OriginRemote = "origin",
    [string]$UpstreamRemote = "upstream",
    [switch]$FetchOnly
)

$ErrorActionPreference = "Stop"

function Invoke-Git {
    param(
        [Parameter(Mandatory = $true)]
        [string[]]$Args
    )

    & git @Args
    if ($LASTEXITCODE -ne 0) {
        throw "git $($Args -join ' ') failed"
    }
}

$repoRoot = (& git rev-parse --show-toplevel).Trim()
if (-not $repoRoot) {
    throw "Not inside a git repository."
}

$status = (& git status --porcelain)
if ($status) {
    throw "Working tree must be clean before starting an upstream sync."
}

Invoke-Git -Args @("fetch", "--prune", $OriginRemote, $BaseBranch)
Invoke-Git -Args @("fetch", "--prune", $UpstreamRemote, $BaseBranch)

$originRef = "$OriginRemote/$BaseBranch"
$upstreamRef = "$UpstreamRemote/$BaseBranch"
$originSha = (& git rev-parse $originRef).Trim()
$upstreamSha = (& git rev-parse $upstreamRef).Trim()

$counts = (& git rev-list --left-right --count "$upstreamRef...$originRef").Trim().Split("`t")
$upstreamAhead = [int]$counts[0]
$originAhead = [int]$counts[1]

Write-Host "Repository root: $repoRoot"
Write-Host "Origin ref:      $originRef ($originSha)"
Write-Host "Upstream ref:    $upstreamRef ($upstreamSha)"
Write-Host "Ahead/behind:    upstream+$upstreamAhead origin+$originAhead"

if ($originSha -eq $upstreamSha) {
    Write-Host "Origin already matches upstream."
    return
}

if ($FetchOnly) {
    Write-Host "Fetch-only mode requested. No sync branch created."
    return
}

$currentBranch = (& git branch --show-current).Trim()
if ($currentBranch -ne $BaseBranch) {
    Invoke-Git -Args @("switch", $BaseBranch)
}

Invoke-Git -Args @("pull", "--ff-only", $OriginRemote, $BaseBranch)

$branchName = "upstream-sync/$((Get-Date).ToUniversalTime().ToString('yyyy-MM-dd'))-$($upstreamSha.Substring(0, 7))"
Invoke-Git -Args @("switch", "-C", $branchName, $originRef)

& git merge --no-ff --no-edit $upstreamRef
if ($LASTEXITCODE -ne 0) {
    throw "Merge produced conflicts. Resolve them on $branchName and continue with a dedicated sync PR."
}

Write-Host ""
Write-Host "Created sync branch: $branchName"
Write-Host "Next steps:"
Write-Host "  git push origin $branchName"
Write-Host "  gh pr create --base $BaseBranch --head $branchName"
