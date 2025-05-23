POD2README(1)	      User Contributed Perl Documentation	 POD2README(1)


NAME
       pod2readme - Intelligently generate a README file from POD

USAGE
	   pod2readme [-cfho] [long options...] input-file [output-file] [target]

	       Intelligently generate a README file from POD

	       -t --target     target type (default: 'readme')
	       -f --format     output format (default: 'text')
	       -b --backup     backup output file
	       -o --output     output filename (default based on target)
	       -c --stdout     output to stdout (console)
	       -F --force      only update if files are changed
	       -h --help       print usage and exit

SYNOPSIS
	   pod2readme -f markdown lib/MyApp.pm

DESCRIPTION
       This utility will use Pod::Readme to extract a README file from a POD
       document.

       It works by extracting and filtering the POD, and then calling the
       appropriate filter program to convert the POD to another format.

OPTIONS
   "--backup"
       By default, "pod2readme" will back up the output file. To disable this,
       use the "--no-backup" option.

   "--output"
       Specifies the name of the output file. If omitted, it will use the
       second command line argument, or default to the "--target" plus the
       corresponding extension of the "--format".

       For all intents, the default is README.

       If a format other than "text" is chosen, then the appropriate extension
       will be added, e.g. for "markdown", the default output file is
       README.md.

   "--target"
       The target of the filter, which defaults to "readme".

   "--format"
       The output format, which defaults to "text".

       Other supposed formats are "github", "html", "latex", "man",
       "markdown", "pod", "rtf", and "xhtml". You can also use "gfm" instead
       of "github". Similary you can use "md" for "markdown".

   "--stdout"
       If enabled, it will output to the console instead of "--output".

   "--force"
       By default, the README will be generated if the source files have been
       changed.  Using "--force" will force the file to be updated.

       Note: POD format files will always be updated.

   "--help"
       Prints the usage and exits.

SEE ALSO
       pod2text, pod2markdown.

perl v5.34.0			  2018-10-31			 POD2README(1)
