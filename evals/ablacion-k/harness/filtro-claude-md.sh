#!/usr/bin/env bash
# filtro-claude-md.sh < CLAUDE.md > claude-md.md — E1: quita la sección «## Memoria de sesiones»
# (hasta la siguiente cabecera ## o el final) y la cabecera que apunta a la KB.
awk '/^## Memoria de sesiones/{skip=1; next} /^## /{skip=0} !skip' | sed '/^> .*wisdom-paul.*exo/d'
