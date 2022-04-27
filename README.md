## SRaft-RS
A simple implementation of Raft.

Warning at the moment it is very very incomplete, it does the absolute basics to understand theory only. It can only tolerate network partitions not 
process crashes. This is due to lack of persistance on the log. So yeah this is something hacked together obviously for my personal learning and you should NEVER USE THIS. 

### TODO 
* PreVote and CheckQuorum is needed badly to ensure liveness in the prescence of network faults.
* Fix loss of messages when no leader is elected
* Critical bug breaks linearizability (the entire protocol pretty much, need to fix this)
