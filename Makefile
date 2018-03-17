NAME := summary
TEX_FILES := $(NAME).tex $(wildcard figures/*.tex)
LATEX := latexmk

.PHONY: clean watch

$(NAME).pdf: $(TEX_FILES)
	$(LATEX) -pdf $(NAME).tex

watch: clean
	$(LATEX) -pdf -pvc $(NAME).tex

clean:
	$(LATEX) -C $(NAME).tex
	rm -rf *.fls *.fdb_latexmk _minted-paper/ *.bbl *.pyg *.latexmain
