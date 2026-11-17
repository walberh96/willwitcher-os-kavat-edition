# Your aliases and functions. Every terminal reads this file; it is yours to change.
# These two come with the system: change them, or take them out.

# port-kill 3000 stops whatever listens on that port, and says what it was.
# port-kill alone lists the ports in use, each with its program.
# A program that is not yours is only named, and stopped, with sudo.
port-kill() {
  local port=$1 line program
  local -a lines as match mbegin mend
  local -A programs
  if [[ -z $port ]]; then
    ss -Htulnp | while read -r line; do
      program='not yours: sudo ss -tulnp names it'
      [[ $line =~ '\("([^"]+)",pid=([0-9]+)' ]] && program="$match[1] ($match[2])"
      # the fifth word is the address with its port, the first tcp or udp
      print -r -- "${${${=line}[5]}##*:}/${${=line}[1]}"$'\t'"$program"
    done | sort -n | uniq | column -t -s $'\t'
    return
  fi
  if [[ $port != <1-65535> ]]; then
    print -u2 "port-kill: $port is not a port"
    return 2
  fi
  lines=( ${(f)"$(ss -Htulnp "sport = :$port")"} )
  if (( ! $#lines )); then
    print "port-kill: nothing listens on port $port"
    return 1
  fi
  if [[ $lines != *pid=* ]]; then
    print "port-kill: what listens on port $port is not yours: stopping it needs sudo"
    as=(sudo)
    lines=( ${(f)"$(sudo ss -Htulnp "sport = :$port")"} )
  fi
  # every process that holds the port, by its number
  for line in $lines; do
    while [[ $line =~ '\("([^"]+)",pid=([0-9]+)' ]]; do
      programs[$match[2]]=$match[1]
      line=${line#*pid=$match[2]}
    done
  done
  if (( ! $#programs )); then
    print -u2 "port-kill: what listens on port $port could not be named"
    return 1
  fi
  # kill -9 in the place of kill stops what does not want to stop
  for line in ${(kn)programs}; do
    if $as kill $line; then
      print "port-kill: stopped $programs[$line] ($line), which listened on port $port"
    else
      print -u2 "port-kill: $programs[$line] ($line) could not be stopped"
    fi
  done
}

# y opens Yazi and, on leaving it, leaves this terminal in the folder Yazi was in.
# It is the wrapper Yazi's own documentation gives. Leaving with Q keeps the folder you
# started in.
y() {
  local tmp="$(mktemp -t "yazi-cwd.XXXXXX")" cwd
  command yazi "$@" --cwd-file="$tmp"
  IFS= read -r -d '' cwd < "$tmp"
  [[ -n $cwd && $cwd != $PWD && -d $cwd ]] && builtin cd -- "$cwd"
  command rm -f -- "$tmp"
}
