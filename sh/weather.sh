#!/usr/bin/sh
req=$(curl -s wttr.in/$1?format=%c+%t+%w+%m+%p)
bar=$(echo $req)
echo $bar
