#!/bin/bash

# This script is from https://github.com/nomlab/tools/blob/master/bin/prodargs

# Get optarg by name
#
# Usage:
#   get_optarg "r" comm -a -t 1 -r 1 -s 1
#
# Result:
#   1
#
function get_optarg() {
  local optname="$1"
  shift

  while [ $# -gt 0 ]
  do
    case "$1" in
      -$optname)
        echo "$2"
        return
        ;;
    esac
    shift
  done
}

# Replace embedded %X style keys by optargs
#
# Usage:
#   replace_optargs comm -a -l logfile-t%t-r%r-s%s.log -t 1 -r 2 -s 3
#
# Result:
#   comm -a -l logfile-t1-r2-s3.log -t 1 -r 2 -s 3
#
function replace_optargs() {
  local before="$@"
  local after=

  while [ $# -gt 0 ]
  do
    # find %X style keys in "$1": "a-%b-%c-d" → "b\nc\n"
    local optnames=$(echo "$1" | sed 's/[^%]*%\(.\)/\1\n/g' | sed '$d')

    # replace %Xs in current_arg
    local current_arg="$1"
    for name in $optnames
    do
      optarg=$(get_optarg "$name" $before)
      current_arg=$(echo "$current_arg" | sed "s!%$name!$optarg!g")
    done
    after="$after $current_arg"
    shift
  done
  echo "$after"
}

# Expand complex sequence of numbers
#
# Usage:
#   sequence "1,3,5-7"
#
# Result:
#   1 3 5 6 7
#
function sequence() {
  local seq="$1"

  for e in $(sed 's/,/ /g' <<<"$seq"); do
    case "$e" in
      [0-9]*-[0-9]*)
        head=$(cut -f1 -d- <<<"$e")
        tail=$(cut -f2 -d- <<<"$e")
        seq -s ' ' "$head" "$tail"
        ;;
      *)
        echo "$e"
        ;;
    esac
  done | tr '\n' ' '
  echo "" # add LF
}

# Guess type of argument
#
# Note: It can't tell "$1" is undefined or ""
# So caller must check if "$1" is defined.
#
function guess_arg_type() {
  case "$1" in
    -*)
      echo "OPT"
      ;;
    [0-9]*[,-][0-9]*)
      echo "ARG_SEQ"
      ;;
    *)
      echo "ARG"
      ;;
  esac
}

# Expand option args into individual ones
#
# Usage:
#   expand_optargs -t 1,3,5-7
#
# Result:
#   -t 1
#   -t 3
#   -t 5
#   -t 6
#   -t 7
#
function expand_optargs() {
  local opt_key="$1"
  local opt_seq="$2"

  for v in $(sequence "$opt_seq"); do
    echo "$opt_key $v"
  done
}

# Expand product of multiple options
#
# Usage:
#   product_optargs comm -a -t 1-3 -r 1,5 -s 1,5
#
# Result:
#   comm -a -t 1 -r 1 -s 1
#   comm -a -t 1 -r 1 -s 5
#   comm -a -t 1 -r 5 -s 1
#   comm -a -t 1 -r 5 -s 5
#   comm -a -t 2 -r 1 -s 1
#   comm -a -t 2 -r 1 -s 5
#   comm -a -t 2 -r 5 -s 1
#   comm -a -t 2 -r 5 -s 5
#   comm -a -t 3 -r 1 -s 1
#   comm -a -t 3 -r 1 -s 5
#   comm -a -t 3 -r 5 -s 1
#   comm -a -t 3 -r 5 -s 5
#
function product_optargs() {
  while [ $# -gt 0 ]
  do
    local args="$1"
    shift

    if [ $# -ge 1 ]; then
      local t1=$(guess_arg_type "$args")
      local t2=$(guess_arg_type "$1")

      case "$t1,$t2" in
        OPT,ARG*)
          local args=$(expand_optargs "$args" "$1")
          shift
          ;;
      esac
    fi
    local prod=$(cartesian_product "$prod" "$args")
  done
  echo "$prod"
}

# Expand product of multiple sequences
#
# Usage:
#  cartesian_product "-t 1
#  -t 2" "-s 1
#  -s 2
#  -s 3"
#
# Result:
#  -t 1 -s 1
#  -t 1 -s 2
#  -t 1 -s 3
#  -t 2 -s 1
#  -t 2 -s 2
#  -t 2 -s 3
#
# Another usage:
#  n=$(expand_optargs -t 1-3)
#  r=$(expand_optargs -r 1,5)
#  s=$(expand_optargs -s 1,5)
#  cartesian_product some_benchmark "$n" "$r" "$s"
#
# Result:
#  some_benchmark -t 1 -r 1 -s 1
#  some_benchmark -t 1 -r 1 -s 5
#  some_benchmark -t 1 -r 5 -s 1
#  some_benchmark -t 1 -r 5 -s 5
#  some_benchmark -t 2 -r 1 -s 1
#  some_benchmark -t 2 -r 1 -s 5
#  some_benchmark -t 2 -r 5 -s 1
#  some_benchmark -t 2 -r 5 -s 5
#  some_benchmark -t 3 -r 1 -s 1
#  some_benchmark -t 3 -r 1 -s 5
#  some_benchmark -t 3 -r 5 -s 1
#  some_benchmark -t 3 -r 5 -s 5
#
function cartesian_product() {
  local seq1="$1"
  local seq2="$2"
  shift 2

  if [ $# -gt 0 ]; then
    local ptmp=$(cartesian_product "$seq1" "$seq2")
    cartesian_product "$ptmp" "$@"
    return
  fi

  local LF=$'\n'
  while read -d "$LF" e1; do
    while read -d "$LF" e2; do
      echo "$e1 $e2"
    done <<<"$seq2"
  done <<<"$seq1"
}

################################################################
## main

# Usage: prodargs [OPTIONS] [--] PARAMS...
# options:
#  -n: show what happend by PARAMS

################
# Parse OPTIONS

while getopts "nh" flag
do
  case $flag in
    # getopts sets '?' to flag on error.
    n)    OPT_DRYRUN=1
          ;;
    \?|h) OPT_ERROR=1
          ;;
  esac
done
shift $(( $OPTIND - 1 ))
OPTIND=

################
# change command if DRY-RUN

if [ "$OPT_DRYRUN" = 1 ];then
  command="cat"
else
  command="sh -v"
fi

################
# Do it

args=$(product_optargs "$@")

while read -r line; do
  replace_optargs $line
done <<<"$args" | $command
