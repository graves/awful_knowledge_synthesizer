#!/opt/homebrew/bin/nu

fd --maxdepth 1 -e yaml . |  lines | enumerate | each { |file|
	print $file.item

	open --raw $"($file.item)"
		| encode utf-8 
		| decode utf-8  
		| str replace -a -r '\x0c' ""
		| str replace -a -r '<think>\n    \n    </think>\n    \n    ' ''
		| str replace -a -r '\u2019' "'"
		| str replace -a -r '\u201c' '"'
		| str replace -a -r '\u201d' '"'
		| to json | from json
		| save -f $"($file.item)"

	expand -t 2 $file.item | save -f $"fixed-($file.item)"
	mv $"fixed-($file.item)" $file.item
}
