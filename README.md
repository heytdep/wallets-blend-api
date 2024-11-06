# Blend Vault API

Follows the blend vault spec.

## Components

1. `/zephyr`. This is the lower level worker that interacts with  blend. The API itself is alse very simple and just feeds off
strings and integers, abstracts any kind of blend specific logic (such as reserves or tokens idxs), and gives back a simulated
`Transaction` object ready to be signed submitted to the network. 
2. `/api`. A simple ergonomic wrapper around the zephyr worker. This offers a dedicated rest api that is more ergonomic to
interact with. This aids the fact of not needing a client side sdk that adds dependency requirements and more maintenance 
headaches.
