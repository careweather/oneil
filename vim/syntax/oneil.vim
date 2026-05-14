" Vim syntax file
" Language:        Oneil Design Specification Language
" Maintainer:      Patrick Walton & Tim Whiting
" Latest Revision: May 13, 2026

if exists("b:current_syntax")
  finish
endif

"----------------------------------------------------------------/
"----------------------------------------------------------------/
"----------------------------------------------------------------/
syn include @tex syntax/tex.vim
autocmd BufNewFile,BufRead *.tex syntax sync fromstart


"----------------------------------------------------------------/
"  Parameters
"----------------------------------------------------------------/

" Piecewise branches: { expr if cond }
syn match oneilPiecewiseKey /{/ contained
syn match oneilConditionKey /if/ contained
syn region oneilPiecewiseN start=/\_^\s\+{/ end=/\_$/ contains=oneilPiecewiseKey,oneilConditionKey

" Preamble: label text before the first colon
syn match oneilBreakpoint /^\*\{1,2}/ contained
syn match oneilPerformance /\$/ contained
syn match oneilName /^[^:(\[]*/ contained contains=oneilBreakpoint,oneilPerformance
syn match oneilOptionKeys /[\[\]()]/ contained
syn region oneilParameterPreamble start=/^\(\*\{1,2}\s*\)\?\(\$\s*\)\?\w/ end=/:/me=e-1 contained contains=oneilName,oneilOptionKeys

" matchgroup=oneilRenderBrace gives every { } inside the render name
syn region oneilRenderName matchgroup=oneilRenderBrace start=/\s*{/ end=/}/ contained contains=@tex
" Defining identifier: after ':' or after the closing '}' of a render name
syn match oneilID /\%([:}]\)\@<=\s*\w\+\s*\ze=/ contained skipwhite

" Submodel alias: the 'bar' in foo.bar — matched inside oneilSubmodule
syn match oneilSubmodelIdent /\<[A-Za-z_][A-Za-z0-9_]*\>/ contained
" Plain identifier in RHS expression (fallback, listed last in contains)
syn match oneilExprIdent /\<[A-Za-z_][A-Za-z0-9_]*\>/ contained

" RHS expression components
syn match oneilMax /.max/ contained
syn match oneilMin /.min/ contained
syn match oneilSubmodule /\.\<[_a-zA-Z]\+\>\(\s*(\)\@!/ contained contains=oneilMin,oneilMax,oneilSubmodelIdent
syn match oneilFunction /\w\+\ze(/ contained
syn match oneilExtremesDelimiter /|/ contained
syn match oneilNumber /\<\d\+\%(\.\d\+\)\?\%([eE][+-]\?\d\+\)\?\>/ contained
syn region oneilString start=/'/ end=/'/ contained contains=oneilStringEscape
syn match oneilStringEscape /\\./ contained
syn keyword oneilConstant true false inf contained
syn keyword oneilLogicalOp and or not contained
syn match oneilOperator /--\|\/\/\|<=\|>=\|==\|!=\|[+\-*\/%^<>]/ contained

syn match oneilAssignment /\%(\w\+\s*=\s*\)\@<=[^:]*/ contained contains=oneilSubmodule,oneilExtremesDelimiter,oneilFunction,oneilPiecewiseKey,oneilConditionKey,oneilNumber,oneilString,oneilConstant,oneilLogicalOp,oneilOperator,oneilExprIdent

" Units: text after the second colon
syn match oneilUnit /\%(=.*:\)\@<=.*$/ contained skipwhite

" Full parameter region — error highlight means any malformed element shows through
syn match oneilParameterKeys /[:=]/ contained
syn region oneilParameter start=/\_^\v(\n)@!(#)@!(submodel)@!(reference)@!(import)@!(test)@!(section)@!(\s)@![^:=]*:/ end=/\_$\n/ contains=oneilParameterPreamble,oneilRenderName,oneilAssignment,oneilUnit,oneilID,oneilParameterKeys


"----------------------------------------------------------------/
"  Design files
"----------------------------------------------------------------/

syn keyword oneilDesignKeyword design contained
syn match oneilDesignTarget /\(\w[/\\]\)*\w\+/ contained
syn region oneilDesignDecl start=/\_^design\>/ end=/\_$/ transparent contains=oneilDesignKeyword,oneilDesignTarget

syn keyword oneilApplyKeyword apply to contained
syn region oneilApplyDecl start=/\_^apply\>/ end=/\_$/ transparent contains=oneilApplyKeyword,oneilDesignTarget

syn keyword oneilSubmodelKeyword submodel reference as contained
syn region oneilSubmodelDecl start=/\_^\(submodel\|reference\)\>/ end=/\_$/ transparent contains=oneilSubmodelKeyword,oneilDesignTarget

" Design body: `id[.submodel] = value [:unit]`
syn match oneilValueID /\w\+\.*\w*\s*\ze=/ contained skipwhite
syn region oneilDesignValue start=/\_^\v(\s|\n|#|(import|test|section|apply|submodel|reference|design)\>)@!(\w+(\.\w+)?\s*\=)@=/ end=/\_$/ contains=oneilUnit,oneilParameterKeys,oneilRenderName,oneilExtremesDelimiter,oneilValueID,oneilAssignment


"----------------------------------------------------------------/
"  Tests
"----------------------------------------------------------------/

syn keyword oneilTestKeys test contained
syn match oneilArgumentKeys /[{}]/ contained
syn match oneilTestPreamble /\_^\*\{0,2}\s*\w[^\n]*:/me=e-1 contained contains=oneilTestKeys,oneilArgumentKeys,oneilBreakpoint
syn match oneilTestDelimiters /[:]/ contained
syn region oneilTestExpression start=/:/ end=/\_$/ contained contains=oneilSubmodule,oneilFunction,oneilNumber,oneilString,oneilConstant,oneilLogicalOp,oneilOperator,oneilExprIdent
syn region oneilTest start=/\_^\(\*\{1,2}\s*\)\{0,1}test/ end=/\_$/ contains=oneilTestPreamble,oneilTestDelimiters,oneilTestExpression


"----------------------------------------------------------------/
"  Includes
"----------------------------------------------------------------/

syn keyword oneilIncludeKeyword use as contained import
syn match oneilModule /\<[A-Za-z_][A-Za-z0-9_/]*\>/ contained
syn match pythonModule /\<[A-Za-z_][A-Za-z0-9_./]*\>/ contained

syn region oneilIncludeLine start=/\_^use/ end=/\_$/ transparent contains=oneilIncludeKeyword,oneilModule
syn region oneilIncludeLine start=/\_^import/ end=/\_$/ transparent contains=oneilIncludeKeyword,pythonModule
syn region oneilIncludeLine start=/\_^from/ end=/\_$/ transparent contains=oneilIncludeKeyword,oneilModule


"----------------------------------------------------------------/
"  Notes
"----------------------------------------------------------------/

" Math spans — $$ display before $; keepend prevents tex.vim eating the closing $
syn region oneilNoteMathBlock  start=/\$\$/ end=/\$\$/  contained keepend contains=@tex
syn region oneilNoteMathInline start=/\$\(\$\)\@!/ end=/\$/ contained keepend contains=@tex

" {{var:equation}} / {{var:value}} placeholder interpolation
syn match oneilNoteInterp /{{\w\+:\%(equation\|value\)}}/ contained contains=oneilNoteInterpType,oneilNoteInterpVar
syn match oneilNoteInterpType /\%(equation\|value\)/ contained containedin=oneilNoteInterp
syn match oneilNoteInterpVar /\w\+/ contained containedin=oneilNoteInterp

syn keyword oneilNoteTodo containedin=oneilNote,oneilMultiLineNote contained TODO FIXME NOTE

" Multi-line note: ~~~...~~~ block (must precede oneilNote; no @markdown — conflicts with fenced-code rules)
syn region oneilMultiLineNote start=/^\s*\~\~\~\~*\s*$/ end=/^\s*\~\~\~\~*\s*$/ fold keepend contains=oneilNoteMathBlock,oneilNoteMathInline,oneilNoteInterp,oneilNoteTodo

" Single/indented note: ends at the next column-0 token or ~~~ line
syn region oneilNote start=/\_^\s\+\(\~\~\~\)\@!\(\S\)\@=\({\)\@!/ end=/\n*\(\n\_^\w\|\n\_^#\|\n\_^\*\|\n\_^\$\|\n\s*\~\~\~\)\@=/me=e-4 fold keepend contains=oneilNoteMathBlock,oneilNoteMathInline,oneilNoteInterp,oneilNoteTodo
syn sync fromstart
" setlocal (not set) so this only affects the current Oneil buffer, not all buffers.
setlocal foldmethod=syntax


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
"  Highlight links
"----------------------------------------------------------------/

let b:current_syntax = "oneil"

"  Top-level declarations
hi def link oneilIncludeKeyword     Keyword
hi def link oneilDesignKeyword      Keyword
hi def link oneilApplyKeyword       Keyword
hi def link oneilSubmodelKeyword    Keyword
hi def link oneilSectionKeyword     Keyword
hi def link oneilDesignTarget       Function
hi def link oneilModule             Function
hi def link pythonModule            Function

"  Parameter structure
hi def link oneilParameter          Error
hi def link oneilParameterKeys      Delimiter
hi def link oneilName               String
hi def link oneilRenderName         String
hi def link oneilRenderBrace        Delimiter
hi def link oneilID                 Function
hi def link oneilUnit               Type
hi def link oneilBreakpoint         Error
hi def link oneilPerformance        Operator
hi def link oneilOptionKeys         Keyword

"  Expressions
hi def link oneilSubmodelIdent      Type
hi def link oneilSubmodule          Identifier
hi def link oneilFunction           PreProc
hi def link oneilExtremesDelimiter  Keyword
hi def link oneilExprIdent          Normal
hi def link oneilNumber             Number
hi def link oneilString             String
hi def link oneilStringEscape       SpecialChar
hi def link oneilConstant           Constant
hi def link oneilLogicalOp          Keyword
hi def link oneilOperator           Operator
hi def link oneilPiecewiseKey       Keyword
hi def link oneilConditionKey       Keyword

"  Design values
hi def link oneilValueID            Function
hi def link oneilDesignValue        Error

"  Tests
hi def link oneilTestKeys           Keyword
hi def link oneilArgumentKeys       Keyword
hi def link oneilTestPreamble       PreProc
hi def link oneilTestDelimiters     Delimiter
hi def link oneilTest               Error

"  Notes
hi def link oneilNote               Comment
hi def link oneilMultiLineNote      Comment
hi def link oneilNoteTodo           Todo
hi def link oneilNoteMathBlock      Special
hi def link oneilNoteMathInline     Special
hi def link oneilNoteInterp         Special
hi def link oneilNoteInterpVar      Identifier
hi def link oneilNoteInterpType     Keyword

"  Comments / sections
hi def link oneilComment            SpecialComment
hi def link oneilSectionHeader      SpecialComment
