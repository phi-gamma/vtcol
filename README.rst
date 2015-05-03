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

Also, in order to view a scheme’s definition, for instance in order to verify
that **vtcol** parses it correctly, specify the ``--dump`` option.

::

    vtcol --dump default
    vtcol --dump ./schemes/solarized

This will print the list of color definitions as dictated by the scheme; if the
specified name does not resolve to a pre-defined scheme it will be interpreted
as a file name instead.

:: _setcolors:  https://github.com/.../linux-vt-setcolors

