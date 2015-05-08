###############################################################################
                                     VTCOL
###############################################################################

Change the color scheme of the virtual Linux console. Inspired by the
setcolors_ utility.

Usage
-----
**vtcol** knows two ways of loading a color scheme: Either by picking the
definitions for a set of predefined schemes or by loading it from a definition
file. The latter accepts input in the format supported by setcolors_. NB not
much effort has been put into ensuring compliance so YMMV. Check the
subdirectory ``./schemes`` in the **vtcol** tree for examples.

Three color schemes are predefined:

    * ``default``          the default color scheme of the Linux console.
    * ``solarized``        the Solarized_ color scheme, dark version.
    * ``solarized_light``  the Solarized_ color scheme, light version.

Invoke **vtcol** with the ``--scheme`` option specifying the scheme of your
choice:

::

    vtcol --scheme solarized_light

In order to view the available schemes, use the ``--list`` option. Should the
scheme specified not resolve to one of the predefined ones, **vtcol** will fall
back to interpreting the name as that of a file. Likewise, loading a scheme
directly from a definition file is accomplished by specifying the ``--file``
argument.

::

    vtcol --file ./schemes/solarized

Instead of an actual scheme or file name, these parameters accept ``-``
as an argument in order to read from ``stdin``.

Also, in order to view a scheme’s definition, for instance in order to verify
that **vtcol** parses it correctly, specify the ``--dump`` option.

::

    vtcol --dump default
    vtcol --dump ./schemes/solarized

This will print the list of color definitions as dictated by the scheme; if the
specified name does not resolve to a pre-defined scheme it will be interpreted
as a file name instead.

Building
--------
The **vtcol** repository aims at compliance with the standard Rust toolchain.
Consequently, the project is packaged using Cargo_. In order to compile a
binary, run

::

    cargo build

In the project root. This should get you a ``vtcol`` binary.

Background
----------
The default palette that comes with a Linux terminal was inherited from a long
history of virtual console implementations. The colors assigned are chosen for
totally valid pragmatic reasons. However, the palette may not harmonize with
everybody’s taste. Unfortunately, the console can’t be themed easily: One needs
to invoke a special ``ioctl(2)`` with the colors prepared in binary form in
order for the kernel to switch the palette.

**vtcol** attempts at facilitating the themability of the console by means of a
simple plain text input format. The very popular themes from the Solarized_
family are included as predefined palettes; the same is true of the Linux
default palette, so they can be conveniently restored when experimenting.

An implementation in C which **vtcols** draws much inspiration from is
available in the setcolors_ utility. **vtcols** itself is implemented in Rust
instead; a public repository is available on Github_. The author uses the
original setcolors_ a lot, primarily inside his custom initramfs. The primary
motivations of writing **vtcols** stems from curiosity as to how the same goal 
might be achieved using more modern tools.

About
-----
**vtcols** was written mostly during day-long train rides between Tübingen and
Dresden, so expect the commit history to exhibit a certain lack of
continuity. Its author_ is Philipp Gesang; see the Bitbucket
(author-bb_) and Github (author-gh_) pages.

The **vtcol** source code is available from the `canonical repository`_.

**vtcol** is redistributable under the terms of the
`GNU General Public License`_ version 3 (exactly). The full text of the
license is contained in the file ``COPYING`` in the root of the
repository. Email the author_ if you wish to use it under a different
license, there’s a non-zero chance that you might convince me.

Patches or suggestions welcome.

.. _setcolors:                  https://github.com/EvanPurkhiser/linux-vt-setcolors
.. _Solarized:                  http://ethanschoonover.com/solarized
.. _Github:                     https://github.com/phi-gamma/vtcols
.. _author:                     mailto:phg@phi-gamma.net
.. _author-bb:                  https://bitbucket.org/phg
.. _author-gh:                  https://github.com/phi-gamma
.. _Cargo:                      https://github.com/rust-lang/cargo
.. _GNU General Public License: http://www.gnu.org/licenses/gpl.txt
.. _canonical repository:       https://github.com/phi-gamma/vtcol

