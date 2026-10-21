$oldUserProfile = $env:USERPROFILE
try {
    $env:USERPROFILE = 'C:\dev\tools\rust'
    topcoat asset bundle --package server
}
finally {
    $env:USERPROFILE = $oldUserProfile
}