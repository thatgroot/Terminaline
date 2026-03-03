set app_path to POSIX path of (path to me)
set bin_path to app_path & "Contents/Resources/terminaline"
do shell script "open -a Terminal " & quoted form of bin_path
