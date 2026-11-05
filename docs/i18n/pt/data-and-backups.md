# Dados e cópias de segurança

## Onde ficam os ficheiros {#where-files-are-kept}

| | Linux | Windows | macOS |
|---|---|---|---|
| Definições (`config.json`) | `~/.config/fausteplayer/` | `%APPDATA%\Fauste\Fauste Player\config\` | `~/Library/Application Support/org.Fauste.Fauste-Player/` |
| Biblioteca, listas, cartuchos e sessão (`fauste.db`) | `~/.local/share/fausteplayer/` | `%APPDATA%\Fauste\Fauste Player\data\` | `~/Library/Application Support/org.Fauste.Fauste-Player/` |
| Cache de análise | `~/.cache/fausteplayer/` | `%LOCALAPPDATA%\Fauste\Fauste Player\cache\` | `~/Library/Caches/org.Fauste.Fauste-Player/` |
| Registos e relatórios de falhas | `~/.local/share/fausteplayer/logs/` | `%LOCALAPPDATA%\Fauste\Fauste Player\data\logs\` | `~/Library/Application Support/org.Fauste.Fauste-Player/logs/` |

Os caminhos do Linux seguem as variáveis XDG (`XDG_CONFIG_HOME` e outras)
quando estão definidas.

O snap guarda os seus ficheiros na sua própria pasta: as definições em
`~/snap/fauste-player/current/.config/fausteplayer/`, a base de dados e os
registos em `~/snap/fauste-player/current/.local/share/fausteplayer/`, e a
cache de análise em `~/snap/fauste-player/common/.cache/fausteplayer/`.
`sudo snap remove fauste-player` também os elimina (mantém um instantâneo
durante algum tempo; `--purge` não).

## Modo portátil {#portable-mode}

Defina a variável de ambiente `FAUSTE_HOME` para uma pasta e tudo passa a
ficar guardado aí, em `config/`, `data/`, `cache/` e `logs/`. É útil numa
pen USB, ou para manter configurações separadas lado a lado.

## Ficheiros {#files}

| Ficheiro | Conteúdo |
|---|---|
| `config.json` | Definições, saídas, limiares de análise, afinação avançada |
| `fauste.db` | A biblioteca (faixas, as suas categorias, marcadores manuais, etiquetas, loudness), as listas com as suas marcas de tocada, as páginas de cartuchos e a sessão: para cada leitor a lista visível, as faixas atual e seguinte, o modo, a posição, o volume e as larguras das colunas (por coluna), e a disposição da janela |
| `fauste.db.bak1`, `.bak2`, `.bak3` | Cópias da base de dados (ver abaixo) |
| `window.ron` | O tamanho e a posição da janela principal e se estava maximizada (só no computador; no Wayland é o sistema que escolhe a posição) |

## Gravação automática e cópias de segurança {#autosave-and-backups}

- As alterações são guardadas cerca de um segundo depois de acontecerem, e
  só se escreve o que mudou. Enquanto algo toca, a sessão (posições) é
  atualizada ao mesmo ritmo.
- A base de dados é escrita em transações, por isso um corte de energia
  nunca deixa uma alteração guardada a meio.
- Na primeira gravação de cada sessão, e depois cerca de cada cinco minutos
  enquanto se guardam alterações, a aplicação copia a base de dados para
  `fauste.db.bak1` e desloca as cópias mais antigas para `.bak2` e `.bak3`.
  Reiniciar várias vezes seguidas empurra, por isso, as cópias mais antigas
  para fora.
- Se a base de dados não puder ser lida, é usada a cópia de segurança válida
  mais recente. O ficheiro ilegível é mantido, com o nome alterado para
  `fauste.db.corrupt-<hora>`, para inspeção. Se todas as cópias falharem, a
  aplicação arranca vazia. Arranca sempre.
- Uma cópia que não pode ser usada também é mantida: quando a aplicação
  arranca a partir de outra cópia (ou de uma base de dados vazia), as
  inutilizáveis passam a chamar-se `fauste.db.bak1.corrupt-<hora>` (ou
  `.newer-<hora>`) para que a próxima cópia de segurança nunca as empurre
  para fora.
- Se outra cópia da aplicação estiver a usar a base de dados, esta corre sem
  guardar e di-lo ao arrancar.
- Uma base de dados escrita por uma versão mais recente da aplicação não é
  aberta: é mantida como `fauste.db.newer-<hora>`, e a aplicação arranca a
  partir da cópia mais recente que consiga ler, ou vazia se não houver
  nenhuma, pelo que abrir os dados com uma versão mais antiga nunca os
  destrói.
- `config.json` é guardado como antes: um ficheiro temporário substitui o
  antigo, e as versões anteriores são mantidas como `.bak1`, `.bak2` e
  `.bak3` a seu lado.
- Se a base de dados não puder ser usada de todo numa sessão (por exemplo,
  não pode ser renomeada porque a pasta é só de leitura), a aplicação corre
  sem guardar até ao próximo arranque, e o registo di-lo.

## Listas e sessão de uma versão anterior {#playlists-and-session-from-an-earlier-version}

As versões anteriores guardavam as listas em `playlists.json`, as páginas de
cartuchos em `carts.json` e a sessão em `session.json`. O primeiro arranque
desta versão lê-os (com as suas cópias `.bak`) uma só vez e põe tudo em
`fauste.db`: a biblioteca com os seus marcadores, etiquetas e loudness, as
listas com as suas marcas de tocada, as páginas de cartuchos e a sessão.
Depois, os ficheiros antigos passam a ter `.imported` no fim
(`playlists.json.imported`, `playlists.json.bak1.imported`, e assim por
diante). Já não são usados e pode eliminá-los quando tiver a certeza de que
está tudo lá.

Para voltar a uma versão anterior:

1. Feche a aplicação.
2. Mova `fauste.db`, `fauste.db-wal`, `fauste.db-shm` e todos os
   `fauste.db.bak*` para uma pasta fora da pasta de dados. Não os elimine.
3. Retire `.imported` dos nomes dos ficheiros antigos.
4. Arranque a versão anterior.

Tudo o que foi guardado desde a atualização fica apenas na pasta para onde
moveu os ficheiros: os ficheiros antigos não o têm. Se atualizar de novo mais
tarde, os ficheiros antigos são importados outra vez, desde que não haja
nenhum `fauste.db` nem `fauste.db.bak*` na pasta de dados.

Se um arranque for interrompido durante a importação, o seguinte conclui-a.
Se um dos ficheiros antigos não puder ser lido (demasiado grande, ou escrito
por uma versão mais recente), a aplicação não importa nada e corre sem
guardar essa sessão, pelo que os ficheiros antigos ficam onde estão; o
registo indica o ficheiro. Corrija-o ou remova-o e arranque de novo. Depois
de `fauste.db` existir, os ficheiros JSON antigos nunca mais são importados.

## Recuperação após uma falha {#crash-recovery}

Após uma falha ou um reinício, cada leitor regressa com a sua lista, as
faixas atual e seguinte e a sua posição, mas **em pausa ou parado**. Nada
vai para o ar por si só. Um leitor que estava mesmo no fim da sua faixa
regressa ao início dessa faixa, para que premir Play a toque em vez de a
terminar de imediato.

## Editar `config.json` à mão {#editing-configjson-by-hand}

Feche primeiro a aplicação (se houver áudio no ar, pergunta antes de fechar).
Os valores desconhecidos ou fora do intervalo são corrigidos para o valor
válido mais próximo quando o ficheiro é carregado, e as correções ficam
registadas. As secções `limits` e `tuning` contêm valores avançados (limites
de recursos, temporização do motor) que não estão na janela de Definições.
