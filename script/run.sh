#!/bin/bash

function usage() {
cat << EOT
run.sh -h HOSTFILE -t TIMESLICE -p PROG_NAME -l LOGFILE

Example:
* run.sh -h ./hostfile -t 100 -p example -l \$(date "+%Y%m%d%H%I%M").log

Caution:
1. PROG_NAME must locate same path on all nodes.
2. You must connect ssh to all nodes in hostfile without password or passphrase.
EOT
}

# Parse command line options
while getopts "h:t:p:l:H" o; do
    case "${o}" in
        h)
            HOSTFILE=${OPTARG}
            ;;
        t)
            TIMESLICE=${OPTARG}
            ;;
        p)
            PROG_NAME=${OPTARG}
            ;;
        l)
            LOGFILE=${OPTARG}
            ;;
        H)
            usage
            exit 0
            ;;
        *)
            usage
            exit 1
            ;;
    esac
done

# Check cmdline options are valid.
if ! [ -e $HOSTFILE ]; then
    echo "Not found: $HOSTFILE"
    exit 1
fi
HOSTS=$(cat $HOSTFILE | awk '{print $1}' | tr '\n' ' ' | awk '{$1=$1};1')
HOSTS_CSV=$(echo $HOSTS | sed 's/ /,/g')

if [ -z $TIMESLICE ]; then
    echo "TIMESLICE is not specified"
    exit 1
fi

if [ -z $PROG_NAME ]; then
    echo "PROG_NAME is not specified"
    exit 1
fi

if ! [ -e $PROG_NAME ]; then
    echo "Not found: $PROG_NAME"
    exit 1
fi

if [ -z $LOGFILE ]; then
    echo "LOGFILE is not specified"
    exit 1
fi

# Check command is installed
if which mpirun > /dev/null; then
    MPIRUN=mpirun
else
    echo "Not found mpirun"
    exit 1
fi

if which rapictld > /dev/null; then
    RAPICTLD=rapictld
else
    echo "Not found rapictld"
    exit 1
fi

if which rapi.so > /dev/null; then
    RAPI=$(which rapi.so)
else
    echo "Not found rapi.so"
    exit 1
fi

# Stop this script if command fails
set -e

# Start rapid on each machine
for HOST in $HOSTS; do
    echo "Launch rapid on $HOST"
    ssh -f $HOST killall -q rapid\; rapid -a $(hostname)
done

# Start rapictld
killall -q rapictld || true
echo "Launch rapictld with TIMESLICE=$TIMESLICE, HOSTS=$HOSTS_CSV"
$RAPICTLD -t $TIMESLICE -n $HOSTS_CSV &
rapictld_pid=$!

# Create logfile and write config
mkdir -p "$(dirname $LOGFILE)"
echo "{\"date\":\"$(date "+%Y-%m-%d %H:%I:%M")\",\"program\":\"$PROG_NAME\",\"timeslice\":$TIMESLICE,\"hosts\":\"$HOSTS_CSV\"}" > $LOGFILE

# Run MPI program
echo "Launch $PROG_NAME"
$MPIRUN -x LD_PRELOAD="$RAPI" --hostfile $HOSTFILE $PROG_NAME >> $LOGFILE

# Stop rapid and rapictld
echo "Stop rapictld"
kill $rapictld_pid
for HOST in $HOSTS; do
    echo "Stop rapid on $HOST"
    ssh $HOST killall rapid
done
