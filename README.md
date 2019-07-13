# MCast

A command line tool to listen for or send multicast UDP datagrams.  

This was originally motivated by an occasion where people believed
multicast was enabled on the network but was actually misconfigured.
The tool acts as a means for a user to send test datagrams while a
network admin is performing a packet capture, possibly on a different
host/device to where the mcast tool is being invoked.  

To listen on a multicast address 231.0.3.1 port 4001, on the network
interface identified by 192.168.1.177:

```
$ mcast listen 192.168.1.177 4001 231.0.3.1
``` 

(Fire up several instances of the command in multiple terminals if you wish)

Try to send data to the multicast group by running:

```
$ mcast send 192.168.1.177 4001 231.0.3.1 << END
Lorem ipsum etc, etc
So on and so forth
END
```

If working, the listen commands entered previously should print the exact 
data that was entered in the send

```
Lorem ipsum etc, etc
So on and so forth
```

For the moment mcast only works with IPv4.


