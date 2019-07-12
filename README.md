# MCast

A small diagnostic tool to help determine whether multicast UDP is
functioning.  

For example, to listen on a multicast address 231.0.3.1 port 4001, 
on the network interface identified by 192.168.1.177:

```
$ mcast listen 231.0.3.1 4001 192.168.1.177
``` 

(Fire up several instances of the command in multiple terminals if you wish)

Try to send data to the multicast group by running:

```
$ mcast send 231.0.3.1 4001 192.168.1.177 << END
Lorem ipsum etc, etc
So on and so forth
END
```

If working, the listen commands entered previously should print the exact 
data that was entered in the send command

```
Lorem ipsum etc, etc
So on and so forth
```

For the moment mcast only works with IPv4.


