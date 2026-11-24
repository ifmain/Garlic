# Elevated VPN only. Own every changed route/rule; preserve other VPN providers.
$ErrorActionPreference='Stop'
$ProgressPreference='SilentlyContinue'
$taskData=[Console]::In.ReadToEnd() | ConvertFrom-Json
function Remove-OwnedState($state) {
    if($state.rule -notmatch '^GarlicVPN-Network-[0-9]+$'){return}
    Get-DnsClientNrptRule -ErrorAction SilentlyContinue | Where-Object {$_.Comment -eq $state.rule} |
        Remove-DnsClientNrptRule -Force -ErrorAction SilentlyContinue
    $allRoutes=@(Get-NetRoute -PolicyStore ActiveStore -ErrorAction SilentlyContinue)
    foreach($route in @($state.routes)) {
        $allRoutes | Where-Object {$_.InterfaceIndex -eq $route.index -and $_.DestinationPrefix -eq $route.prefix -and $_.NextHop -eq $route.gateway -and $_.RouteMetric -eq 1} |
            Remove-NetRoute -Confirm:$false -ErrorAction SilentlyContinue
    }
    if($state.dns){
        if(@($state.dns.servers).Count){Set-DnsClientServerAddress -InterfaceIndex $state.dns.index -ServerAddresses @($state.dns.servers) -ErrorAction SilentlyContinue}
        else{Set-DnsClientServerAddress -InterfaceIndex $state.dns.index -ResetServerAddresses -ErrorAction SilentlyContinue}
    }
    if($state.rule -match '^GarlicVPN-Network-[0-9]+$'){Remove-NetFirewallRule -Name $state.rule -ErrorAction SilentlyContinue}
    foreach($nic in @($state.interfaces)) {
        Get-NetIPInterface -AddressFamily $nic.family -InterfaceIndex $nic.index -ErrorAction SilentlyContinue |
            Where-Object {$_.InterfaceAlias -eq 'Garlic'} |
            Set-NetIPInterface -AutomaticMetric $nic.automatic -InterfaceMetric $nic.metric -ErrorAction SilentlyContinue
    }
    Clear-DnsClientCache -ErrorAction SilentlyContinue
}
function Remove-StaleGarlicDnsRules {
    # Only local rules with the exact signature left by a stopped Garlic VPN.
    $livePids=@(Get-Process -ErrorAction Stop | ForEach-Object {$_.Id})
    foreach($rule in @(Get-DnsClientNrptRule)) {
        if($rule.Comment -notmatch '^GarlicVPN-Network-([1-9][0-9]{0,9})$'){continue}
        $previousOwner=0
        if(-not [int]::TryParse($Matches[1],[ref]$previousOwner)){continue}
        $name=[guid]::Empty
        if(-not [guid]::TryParse([string]$rule.Name,[ref]$name)){continue}
        $namespace=@($rule.Namespace)
        $servers=@($rule.NameServers)
        if($namespace.Count -ne 1 -or $namespace[0] -ne '.' -or
            $servers.Count -ne 1 -or [string]$servers[0] -ne '1.1.1.1'){continue}
        if($livePids -contains $previousOwner){continue}
        $rule | Remove-DnsClientNrptRule -Force
    }
}
if($taskData.operation -eq 'activate'){
    $state=$taskData.state
    try {
        $owner=[int]($state.rule -replace '^GarlicVPN-Network-','')
        if(-not(Get-Process -Id $owner -ErrorAction SilentlyContinue)){throw 'VPN startup cancelled'}
        if(Get-DnsClientNrptPolicy -Effective | Where-Object {$_.Namespace -contains '.'}){throw 'An existing catch-all DNS policy prevents VPN DNS ownership'}
        foreach($route in @($state.capture_routes)){
            if(Get-NetRoute -AddressFamily $route.family -InterfaceIndex $route.index -DestinationPrefix $route.prefix -PolicyStore ActiveStore -ErrorAction SilentlyContinue){throw 'A route prevents VPN capture ownership'}
            New-NetRoute -AddressFamily $route.family -InterfaceIndex $route.index -DestinationPrefix $route.prefix -NextHop $route.gateway -RouteMetric 1 -PolicyStore ActiveStore | Out-Null
        }
        $dnsRoute=Find-NetRoute -RemoteIPAddress 1.1.1.1 | Where-Object {$_.CimClass.CimClassName -eq 'MSFT_NetRoute'} | Select-Object -First 1
        if($dnsRoute.InterfaceIndex -ne $state.dns.index){throw 'Another route owns the VPN resolver path'}
        Set-DnsClientServerAddress -InterfaceIndex $state.dns.index -ServerAddresses '1.1.1.1'
        $dns=Add-DnsClientNrptRule -Namespace '.' -NameServers '1.1.1.1' -Comment $state.rule -PassThru
        $state.nrpt=$dns.Name
        Clear-DnsClientCache
        # If the native owner died during this child operation, a final cleanup
        # also removes writes that raced the already prepared crash watcher.
        if(-not(Get-Process -Id $owner -ErrorAction SilentlyContinue)){throw 'VPN startup cancelled'}
        $state | ConvertTo-Json -Depth 5 -Compress
    }catch{Remove-OwnedState $state;throw}
    exit 0
}
if($taskData.operation -eq 'disable'){
    Remove-OwnedState $taskData.state
    try{$done=[Threading.EventWaitHandle]::OpenExisting($taskData.state.cleaned);$null=$done.Set();$done.Dispose()}catch{}
    exit 0
}
if($taskData.operation -ne 'enable'){throw 'Unknown network guard operation'}
$state=@{routes=@();capture_routes=@();interfaces=@();dns=$null;rule=('GarlicVPN-Network-'+[int]$taskData.owner_pid);nrpt=$null;cleaned=('Global\GarlicVPNNetworkCleaned-'+[int]$taskData.owner_pid)}
function Add-OwnedRoute($family,$index,$prefix,$gateway) {
    if(Get-NetRoute -AddressFamily $family -InterfaceIndex $index -DestinationPrefix $prefix -PolicyStore ActiveStore -ErrorAction SilentlyContinue){throw ('An existing route prevents exclusive ownership: '+$prefix)}
    New-NetRoute -AddressFamily $family -InterfaceIndex $index -DestinationPrefix $prefix -NextHop $gateway -RouteMetric 1 -PolicyStore ActiveStore | Out-Null
    $state.routes+=@(@{family=$family;index=[int]$index;prefix=$prefix;gateway=$gateway})
}
try {
    if(@(Get-NetFirewallProfile -PolicyStore ActiveStore | Where-Object {-not $_.Enabled}).Count -gt 0){throw 'IPv6 protection requires the Windows firewall to be enabled'}
    if(Get-DnsClientNrptPolicy -Effective | Where-Object {$_.Namespace -contains '.'}){
        Remove-StaleGarlicDnsRules
        if(Get-DnsClientNrptPolicy -Effective | Where-Object {$_.Namespace -contains '.'}){
            throw 'An existing catch-all DNS policy prevents exclusive VPN DNS ownership'
        }
    }
    $deadline=[DateTime]::UtcNow.AddSeconds(12)
    do {
        $nic4=Get-NetIPInterface -InterfaceAlias Garlic -AddressFamily IPv4 -ErrorAction SilentlyContinue | Select-Object -First 1
        $nic6=Get-NetIPInterface -InterfaceAlias Garlic -AddressFamily IPv6 -ErrorAction SilentlyContinue | Select-Object -First 1
        $adapter=Get-NetAdapter -Name Garlic -ErrorAction SilentlyContinue
        $preferred=Get-NetIPAddress -InterfaceAlias Garlic -AddressFamily IPv4 -ErrorAction SilentlyContinue | Where-Object {$_.AddressState -eq 'Preferred'}
        if($nic4 -and $nic6 -and $adapter.Status -eq 'Up' -and $preferred){break}; Start-Sleep -Milliseconds 200
    }while([DateTime]::UtcNow -lt $deadline)
    if(-not $nic4 -or -not $nic6 -or $adapter.Status -ne 'Up' -or -not $preferred){throw 'Garlic interface did not become ready'}
    $gateway=[Net.IPAddress]::Parse($taskData.gateway)
    if($gateway.AddressFamily -ne [Net.Sockets.AddressFamily]::InterNetwork){throw 'Invalid VPN IPv4 gateway'}
    # Repair stale routes of older Garlic builds on our own adapter only.
    # The native owner gate prevents another new Garlic VPN from running here.
    foreach($prefix in @('0.0.0.0/0','0.0.0.0/1','128.0.0.0/1','::/0','::/1','8000::/1')){
        Get-NetRoute -InterfaceIndex $nic4.InterfaceIndex -DestinationPrefix $prefix -PolicyStore ActiveStore -ErrorAction SilentlyContinue | Remove-NetRoute -Confirm:$false -ErrorAction SilentlyContinue
    }
    foreach($nic in @($nic4,$nic6)) {
        $state.interfaces+=@(@{family=$nic.AddressFamily.ToString();index=[int]$nic.InterfaceIndex;automatic=$nic.AutomaticMetric.ToString();metric=[int]$nic.InterfaceMetric})
        $nic | Set-NetIPInterface -AutomaticMetric Disabled -InterfaceMetric 1
    }
    $physicalIndices=@(Get-NetAdapter -Physical -ErrorAction SilentlyContinue | Where-Object {$_.Status -eq 'Up'} | ForEach-Object {$_.ifIndex})
    $physicalDefault=Get-NetRoute -AddressFamily IPv4 -DestinationPrefix '0.0.0.0/0' -PolicyStore ActiveStore -ErrorAction SilentlyContinue |
        Where-Object {$physicalIndices -contains $_.InterfaceIndex} |
        Sort-Object @{Expression={$_.RouteMetric+$_.InterfaceMetric}} | Select-Object -First 1
    # Resolve the original best path before adding our split-default routes.
    # Existing /32 routes belong to their current owner and are left untouched.
    foreach($cidr in @($taskData.bypass | Sort-Object -Unique)) {
        $ip=$cidr.Split('/')[0];$address=[Net.IPAddress]::Parse($ip)
        if($address.AddressFamily -ne [Net.Sockets.AddressFamily]::InterNetwork){throw 'VPN bootstrap requires IPv4 endpoints'}
        # The loopback /8 is already more specific than the VPN's /1 routes.
        # Sending a local bootstrap proxy to the physical gateway breaks it.
        if([Net.IPAddress]::IsLoopback($address)){continue}
        $prefix=$ip+'/32'
        $existing=@(Get-NetRoute -AddressFamily IPv4 -DestinationPrefix $prefix -PolicyStore ActiveStore -ErrorAction SilentlyContinue)
        if($existing.Count){
            if($existing | Where-Object {$_.InterfaceIndex -eq $nic4.InterfaceIndex}){throw 'A bootstrap route points into Garlic'}
            continue
        }
        $original=Find-NetRoute -RemoteIPAddress $ip -ErrorAction SilentlyContinue | Where-Object {$_.CimClass.CimClassName -eq 'MSFT_NetRoute'} | Select-Object -First 1
        if(-not $original -or $original.InterfaceIndex -eq $nic4.InterfaceIndex){
            if($physicalDefault){$original=$physicalDefault}else{throw 'No independent bootstrap route'}
        }
        # An outer TUN's encrypted uplink is itself captured by our defaults.
        # Bootstrap our own encrypted connections over a physical path instead
        # of creating a circular dependency on that other VPN's transport.
        if($physicalDefault -and $physicalIndices -notcontains $original.InterfaceIndex){$original=$physicalDefault}
        Add-OwnedRoute IPv4 $original.InterfaceIndex $prefix $original.NextHop
    }
    $firewallDeadline=[DateTime]::UtcNow.AddSeconds(20)
    while($true){
        if(-not(Get-Process -Id $taskData.owner_pid -ErrorAction SilentlyContinue)){throw 'VPN startup cancelled'}
        try{
            New-NetFirewallRule -Name $state.rule -DisplayName 'Garlic VPN IPv6 protection' -Direction Outbound -InterfaceAlias Garlic -RemoteAddress @('2000::/3','fc00::/7','fe80::/10') -Action Block -Enabled True -Profile Any | Out-Null
            break
        }catch{
            if($_.FullyQualifiedErrorId -notlike '*0x80070057*' -or [DateTime]::UtcNow -ge $firewallDeadline){throw}
            Start-Sleep -Milliseconds 250
        }
    }
    $oldDns=Get-DnsClientServerAddress -InterfaceIndex $nic4.InterfaceIndex -AddressFamily IPv4
    $state.dns=@{index=[int]$nic4.InterfaceIndex;servers=@($oldDns.ServerAddresses | Where-Object {$_})}
    foreach($prefix in @('::/1','8000::/1')){$state.capture_routes+=@(@{family='IPv6';index=[int]$nic6.InterfaceIndex;prefix=$prefix;gateway='::'})}
    foreach($prefix in @('0.0.0.0/1','128.0.0.0/1')){$state.capture_routes+=@(@{family='IPv4';index=[int]$nic4.InterfaceIndex;prefix=$prefix;gateway=$gateway.IPAddressToString})}
    # The watcher knows the entire cleanup plan before capture is activated.
    $state.routes+=@($state.capture_routes)
    # The watcher also cleans up after a crash, when Rust Drop cannot run.
    $encodedState=[Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes(($state|ConvertTo-Json -Depth 5 -Compress)))
    $readyName='Global\GarlicVPNNetworkReady-'+[int]$taskData.owner_pid
    $ready=New-Object Threading.EventWaitHandle($false,[Threading.EventResetMode]::AutoReset,$readyName)
    $watcher='$ErrorActionPreference="Stop"; function Remove-OwnedState {'+${function:Remove-OwnedState}.ToString()+'}; '+
        '$gate=[Threading.Mutex]::OpenExisting("Global\GarlicVPNNetworkOwner"); '+
        '$done=New-Object Threading.EventWaitHandle($false,[Threading.EventResetMode]::AutoReset,"'+$state.cleaned+'"); '+
        '$ready=[Threading.EventWaitHandle]::OpenExisting("'+$readyName+'"); $null=$ready.Set(); $ready.Dispose(); '+
        'try{$owner=Get-Process -Id '+[int]$taskData.owner_pid+' -ErrorAction SilentlyContinue; if($owner){$owner.WaitForExit()}; '+
        'if(-not $done.WaitOne(0)){$ErrorActionPreference="SilentlyContinue";$state=[Text.Encoding]::UTF8.GetString([Convert]::FromBase64String("'+$encodedState+'"))|ConvertFrom-Json; Remove-OwnedState $state}}finally{$done.Dispose();$gate.Dispose()}'
    $encodedWatcher=[Convert]::ToBase64String([Text.Encoding]::Unicode.GetBytes($watcher))
    $watcherProcess=$null
    $armed=$false
    try{
        $watcherStarted=[Diagnostics.Stopwatch]::StartNew()
        $watcherProcess=Start-Process -FilePath (Join-Path $PSHOME 'powershell.exe') -WindowStyle Hidden -PassThru -ArgumentList @('-NoProfile','-NonInteractive','-EncodedCommand',$encodedWatcher)
        while(-not $ready.WaitOne(200)){
            if($watcherProcess.HasExited){throw ('Garlic network cleanup watcher exited during startup with code '+$watcherProcess.ExitCode)}
            if(-not(Get-Process -Id $taskData.owner_pid -ErrorAction SilentlyContinue)){throw 'VPN startup cancelled'}
            if($watcherStarted.Elapsed.TotalSeconds -ge 180){throw 'Garlic network cleanup watcher did not initialize'}
        }
        $armed=$true
    }finally{
        if($watcherProcess){
            if(-not $armed){try{if(-not $watcherProcess.HasExited){$watcherProcess.Kill()}}catch{}}
            $watcherProcess.Dispose()
        }
        $ready.Dispose()
    }
    $state | ConvertTo-Json -Depth 5 -Compress
}catch{Remove-OwnedState $state; throw}
