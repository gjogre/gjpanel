#!/usr/bin/sh

req=$(curl -s wttr.in/Hyvinkää?format=%c+%t+%w+%m+%p)
#req=$(curl -s wttr.in/Hyvinkää?format="%t+%C+%h+%l+%m+%M+%F")
bar=$(echo $req)
#tooltip=$(echo "$req" | awk 'NR>1 {print $0}')
echo $bar
