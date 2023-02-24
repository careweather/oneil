" Vim syntax file
" Language:         Oneil Design Specification Language
" Maintainer:       Patrick Walton
" Latest Revision:  December 5, 2022

if exists("b:current_syntax")
  finish
endif

"----------------------------------------------------------------/
"----------------------------------------------------------------/
"----------------------------------------------------------------/
syn include @tex syntax/tex.vim
autocmd BufNewFile,BufRead *.tex syntax sync fromstart

"----------------------------------------------------------------/
"  Parameter
"----------------------------------------------------------------/
"  Piecewise Assignment
syn match oneilPiecewiseKey /{/ contained
syn match oneilConditionKey /if/ contained
syn region oneilPiecewiseN start=/\_^\s\+{/ end=/\_$/ contains=oneilPiecewiseKey, oneilConditionKey

"  Preamble - before the first colon
syn match oneilBreakpoint /^\*\{1,2}/ contained
syn match oneilPerformance /\$/ contained
syn match oneilName /^[^:(\[]*/ contained contains=oneilBreakpoint,oneilPerformance
syn match oneilOptionKeys /[\[\]()]/ contained
syn region oneilParameterPreamble start=/^\(\*\{1,2}\s*\)\?\(\$\s*\)\?\w/ end=/:/me=e-1 contained contains=oneilName, oneilOptionKeys

"  ID - between the first colon and equals sign
syn match oneilID /\%(:\)\@<=\s*\w\+\s*\ze=/ contained skipwhite

"  Assignment - between the equals sign and the second colon or the end
syn match oneilMax /.max/ contained
syn match oneilMin /.min/ contained
syn match oneilSubmodule /\.\<[_a-zA-Z]\+\>\(\s*(\)\@!/ contained contains=oneilMin,oneilMax
syn match oneilFunction /\w\+\ze(/ contained
syn match oneilExtremesDelimiter /|/ contained
syn match oneilAssignment /\%(\w\+\s*=\s*\)\@<=[^:]*/ contained contains=oneilSubmodule,oneilExtremesDelimiter,oneilFunction,oneilPiecewiseKey,oneilConditionKey

"  Units - after a second colon
syn match oneilUnit /\%(=.*:\)\@<=.*$/ contained skipwhite

"  Main Region and Separators - parameter is any line not starting with
"  keywords and indents reserved for includes and notes
"  parameter region is highlighted with error, so if any of the elements of a
"  properly constructed parameter are missing or malformed, error will show
"  through
syn match oneilParameterKeys /[:=]/ contained
syn region oneilParameter start=/\_^\v(\n)@!(#)@!(use)@!(import)@!(test)@!(section)@!(\s)@![^:=]*:/ end=/\_$\n/ contains=oneilParameterPreamble,oneilAssignment,oneilUnit,oneilID,oneilParameterKeys

"----------------------------------------------------------------/
"  Design
"----------------------------------------------------------------/

syn match oneilValueID /\w\+\.*\w*\s*\ze=/ contained skipwhite
syn region oneilDesignValue start=/\_^\v(\s\+)@!(\n)@!(#)@!(\s\+)@!(use)@!(import)@!(test)@!(section)@!(\w+\s*\=)@=/ end=/\_$/ contains=oneilUnit,oneilParameterKeys,oneilExtremesDelimiter,oneilValueID,oneilAssignment

"----------------------------------------------------------------/
"  Tests
"----------------------------------------------------------------/
syn keyword oneilTestKeys test contained
syn match oneilArgumentKeys /[{}]/ contained
syn match oneilTestPreamble /\_^\*\{0,2}\s*\w[^\n]*:/me=e-1 contained contains=oneilTestKeys,oneilArgumentKeys,oneilBreakpoint
syn match oneilTestDelimiters /[:]/ contained
syn region oneilTestExpression start=/:/ end=/\_$/ contained contains=oneilSubmodule
syn region oneilTest start=/\_^\(\*\{1,2}\s*\)\{0,1}test/ end=/\_$/ contains=oneilTestPreamble,oneilTestDelimiters,oneilTestExpression


"----------------------------------------------------------------/
"  Includes
"----------------------------------------------------------------/

syn keyword oneilIncludeKeyword use as contained import
syn match oneilModule /\w/ contained
syn match pythonModule /\w/ contained

" Include Regions
syn region oneilIncludeLine start=/\_^use/ end=/\_$/ transparent contains=oneilIncludeKeyword,oneilModule
syn region oneilIncludeLine start=/\_^import/ end=/\_$/ transparent contains=oneilIncludeKeyword,pythonModule
syn region oneilIncludeLine start=/\_^from/ end=/\_$/ transparent contains=oneilIncludeKeyword,oneilModule


"----------------------------------------------------------------/
"  Notes
"----------------------------------------------------------------/
syn keyword oneilNoteTodo containedin=oneilNote contained TODO FIXME NOTE
syn region oneilNote start=/\_^\s\+\(\S\)\@=/ end=/\n*\(\n\_^\w\|\n\_^#\|\n\_^\*\|\n\_^\$\)\@=/me=e-4 fold contains=@tex,oneilNoteTodo
"syn region oneilNote start=/\_^\s\+\(\S\)\@=\({\)\@!/ end=/\n*\(\n\_^\w\|\n\_^#\|\n\_^\*\|\n\_^\$\)\@=/ fold contains=@tex,oneilNoteTodo
syn sync fromstart
set foldmethod=syntax


"----------------------------------------------------------------/
"  Sections
"----------------------------------------------------------------/
syn keyword oneilSectionKeyword section contained
syn region oneilSectionHeader start=/\_^section/ end=/\_$/ transparent contains=oneilSectionKeyword

"----------------------------------------------------------------/
"  Comments
"----------------------------------------------------------------/
syn region oneilComment start=/\_^\s*#/ end=/\_$/


"----------------------------------------------------------------/
"----------------------------------------------------------------/
"----------------------------------------------------------------/
let b:current_syntax = "oneil"

hi def link startEndMarker		Special

"  Includes
hi def link oneilIncludeKeyword		Keyword
hi def link oneilModule			Function
hi def link pythonModule		Function

"  Assignments
hi def link oneilPerformance		Operator	
hi def link oneilParameter		Error
hi def link oneilParameterKeys		Delimiter

"hi def link oneilParameterPreamble	Constant
hi def link oneilID			Identifier
"hi def link oneilAssignment		Keyword
hi def link oneilUnit			Type

hi def link oneilName			String
hi def link oneilBreakpoint		Error
hi def link oneilOptionKeys		Keyword
hi def link oneilSubmodule		PreProc
"hi def link oneilFunction		PreProc
hi def link oneilExtremesDelimiter	Keyword
hi def link oneilValue			Constant

hi def link oneilPiecewiseKey		Keyword
hi def link oneilConditionKey		Keyword
"hi def link oneilPiecewiseN		Keyword

"  Values
hi def link oneilValueID		Identifier
hi def link oneilDesignValue		Error

"  Notes
hi def link oneilNote			Comment
hi def link oneilNoteTodo		Todo

"  Comments
hi def link oneilComment		SpecialComment

"  Sections
hi def link oneilSectionKeyword		Keyword
hi def link oneilSectionHeader		SpecialComment

"  Tests
hi def link oneilTestKeys		Keyword
hi def link oneilArgumentKeys		Keyword
hi def link oneilTestPreamble		Preproc
hi def link oneilTestDelimiters		Delimiter
hi def link oneilTest			Error
