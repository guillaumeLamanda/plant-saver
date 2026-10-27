# Vendored esp-idf-matter

Source: https://github.com/sysgrok/esp-idf-matter
Revision: `3dcc2139f3d135cd5805290ee289807f0d6d9a80`

This snapshot restarts Thread around an operational-dataset replacement. OpenThread
does not restore a newly saved dataset while its role is `Detached`, which is the
state used during Matter commissioning.
