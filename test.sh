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

try 0 0
try 42 42
try 3 "1+2"
try 3 "5-2"
try 3 "5-2+1-1"
try 41 " 12 + 34 - 5 "
try 47 '5+6*7'
try 15 '5*(9-6)'
try 4 '(3+5)/2'
try 10 '-10+20'

echo OK