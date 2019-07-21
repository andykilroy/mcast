# MCast

A command line tool to listen for or send multicast UDP datagrams.  

This was motivated by an occasion where multicast was not functioning on
a network, but it wasn't obvious why.  Since the sending application was
sending UDP datagrams infrequently, it was difficult for a network
engineer to tell why multicast receivers were not receiving datagrams.
It would have been useful for the engineer to have a tool to trigger
sends, or start up listening multicast clients, so that they had
something to test with at the point the network was being configured.

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

When working, the listen commands entered previously should print the
exact data that was entered in the send

```
Lorem ipsum etc, etc
So on and so forth
```

For the moment mcast only works with IPv4.


