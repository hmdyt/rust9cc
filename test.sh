#!/bin/bash

rust9cc="./target/debug/rust9cc"

try() {
    expected="$1"
    input="$2"

    ${rust9cc} "$input" > tmp.s
    cc -o tmp tmp.s
    ./tmp
    actual="$?"

  if [ "$actual" = "$expected" ]; then
    echo "$input => $actual"
  else
    echo "$input => $expected expected, but got $actual"
    exit 1
  fi
}

cargo build

try 0 "0;"
try 42 "42;"
try 3 "1+2;"
try 3 "5-2;"
try 3 "5-2+1-1;"
try 41 " 12 + 34 - 5 ;"
try 47 '5+6*7;'
try 15 '5*(9-6);'
try 4 '(3+5)/2;'
try 10 '-10+20;'
try 1 '1+2+3 == 2*3;'
try 0 '1+2+3 != 2*3;'
try 1 ' 1 < 2;'
try 1 ' 1 <= 2;'
try 0 ' 1 > 2;'
try 0 ' 1 >= 2;'
try 1 '1+2 < 100 == -1 < 2*3;'
try 12 'a = 11; a + 1;'
try 12 'hoge=10;hoge+2;'
try 14 'return 14; 1;'
try 7 'if (1 < 2) return 7; return 14;'
try 14 'if (1 > 2) return 7; return 14;'
try 7 'if (1 < 2) return 7;else return 14;'
try 14 'if (1 > 2) return 7;else return 14; return 0;'
try 13 'x = 0; while (x < 13) x = x + 1; return x;'
try 10 'x = 0; for (i = 0; i < 10; i = i + 1) x = x + 1; return x;'
try 10 'if (1 < 2) {return 10;} else {return 20;}'
try 20 'x = 0; i = 0; while (i < 10) {x = x + 2; i = i + 1;} return x;'
try 10 'x = 0; for (i = 0; i < 10; i = i + 1) {x = x + 1;} return x;'

echo OK