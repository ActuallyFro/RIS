RIS (like RIZZ): Rust-based IRC Server
======================================
An attempt to build an IRC server that works with the Rust-based IRC waRgaming Client (rIRC) and mIRC.

* It OUGHT to comply with the [RFC for IRC Servers](https://www.rfc-editor.org/rfc/rfc2813.txt) -- but it's NOT enforced.

* The Server has minimally been verified with the rIRC and mIRC to be functional for passing messages between clients.

Known Limitations
-----------------
* The server is NOT configurable
  * It starts and listens only on port 6667
  * Only offers the `#Main` channel.

* Ctrl+C/Z/etc. or closing the application window will terminate the server.

* The SERVER CANNOT send messages to multiple clients on the same IP address (i.e., the server does not check for multiple users on the same IP address, and has a terrible set of logic to prevent "double message sending" for a single IP).

Open Source License Note
------------------------
1. The project is licensed under the [Creative Commons Zero v1.0 Universal](https://creativecommons.org/publicdomain/zero/1.0/) license.
2. It aligns to the DoD CIO's [MFR on Software DEvelopement and Open Source Software](https://dodcio.defense.gov/portals/0/documents/library/softwaredev-opensource.pdf) dated 24 Jan 22.
3. This license extends the policy intrepretation of CISA's Open Source Approach. [Details Here](https://github.com/cisagov/development-guide/blob/develop/open-source-policy/policy.md)
4. Specifically, the [CISA-implemented version of CC0 is used.](https://github.com/cisagov/development-guide/blob/develop/LICENSE)
