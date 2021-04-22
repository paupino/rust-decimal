#!/bin/bash

for file in *.csv; do
    if [ -f "$file" ]; then
        name=$(echo "$file" | cut -f 1 -d '.' | awk '{print tolower($0)}')
        op=$(echo "$file" | cut -f 1 -d '_' | awk '{print tolower($0)}')

        echo "gen_test!(test_$name, \"$file\", checked_$op);"
    fi
done
