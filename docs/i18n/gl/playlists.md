# Listas

## Lapelas {#tabs}

Cada reprodutor ten unha fila de lapelas, unha por lista. Todos os
reprodutores ven as mesmas listas; cada reprodutor escolle cal mostra. Un
punto nunha lapela indica onde están as pistas do reprodutor: **vermello**
para a pista en antena, **verde** para a seguinte.

As lapelas comparten o ancho do reprodutor. Un nome que non cabe remata en
«…»; pon o punteiro sobre a lapela para lelo completo. Con moitas listas, as
lapelas deixan de encoller a partir dun ancho mínimo, aparecen frechas nos
extremos da fila e a roda do rato sobre as lapelas desprázaas. A lapela que
escolles, e a que mostra un reprodutor, tráese á vista.

Cambiar de lapela nunca cambia o que está en antena nin o que vai despois.
Cando remata unha pista, o reprodutor continúa na lista que contén esa pista.

As listas créanse, renómeanse e elimínanse en [Configuración](settings.md). A
última lista, e unha lista cunha pista en antena, non se poden eliminar.

## A táboa de pistas {#the-track-table}

![Unha lista: pistas reproducidas atenuadas, a pista en antena en vermello, a pista seguinte en verde, e o pé co tempo que queda](../../images/guide/playlist.png)

| Columna | Contido |
|---|---|
| `#` | Posición, con ceros á esquerda; unha icona substitúea para as pistas actual e seguinte |
| Título | Das etiquetas, ou o nome do ficheiro (`Artista - Título.mp3` divídese). As iconas de repetir e parar despois dunha pista van antes del |
| Artista | Das etiquetas; «Artista descoñecido» cando non hai ningún |
| Álbum | Das etiquetas |
| Data | A data de gravación tal como a garda o ficheiro (`2019`, `2019-05` ou `2019-05-14`, cunha hora se a hai) |
| Xénero | Das etiquetas |
| Dur. | Duración de reprodución, desde o cue-in ata o cue-out (o ficheiro enteiro con **Usar cue-in e cue-out** desactivado) |
| Intro | Canto dura a intro, desde onde a pista empeza a soar ata o seu marcador de intro; baleira cando a pista non ten marcador de intro |
| Ficheiro | O nome do ficheiro, coa súa extensión |

Unha instalación nova mostra `#`, Título, Artista e Dur. As outras columnas
son opcionais; consulta **Escoller as columnas** máis abaixo. Unha pista á que
lle falta un valor mostra unha cela baleira, salvo Artista, que mostra
«Artista descoñecido».

As columnas énchena e manteñen as súas proporcións cando se
redimensiona a xanela; as columnas de texto reciben máis espazo. Arrastra os
separadores da cabeceira para cambiar as proporcións: as columnas á dereita
do separador seguen o punteiro en cada fotograma (repártense o que queda en
proporción ao seu ancho), as da súa esquerda quedan onde están, e os anchos
gárdanse cando soltas. Ningunha columna se fai máis estreita que o seu
mínimo. Os anchos lémbranse por reprodutor; unha columna que mostres despois
comeza co seu ancho predeterminado e as outras manteñen as súas proporcións.

### Escoller as columnas {#choosing-the-columns}

Título e Dur. móstranse sempre. Calquera outra columna pódese mostrar ou
ocultar, e calquera columna, incluídas esas dúas, pódese mover. A lista é a
mesma para todos os reprodutores e listas, e gárdase en `config.json` como
`ui.table_columns`. Tres formas de cambiala:

- **Configuración → Listas → Columnas da táboa:** marca as columnas que
  mostrar; as frechas suben ou baixan unha columna mostrada (léense de
  esquerda a dereita nas táboas). **Columnas predeterminadas** volve a `#`,
  Título, Artista e Dur.
- **Clic dereito nunha cabeceira:** un menú cunha caixa de selección para cada
  columna opcional. Unha columna que mostres aparece no extremo dereito;
  arrástraa desde alí.
- **Arrastrar unha cabeceira** sobre outra: solta na metade esquerda dunha
  cabeceira para poñer a columna antes dela, na metade dereita para poñela
  despois. Soltar noutro sitio non fai nada.

Un nome en `ui.table_columns` que esta versión non coñece ignórase, e un
Título ou Dur. que falte engádese de novo.

Cando se abre a aplicación, cada táboa desprázase para que a pista seguinte
do seu reprodutor quede no medio da táboa (o máis preto que permitan os
extremos da lista). Iso sucede unha vez, ao iniciar, e só cando a pista
seguinte está na lista que mostra a táboa.

Cando un reprodutor pasa a outra pista, a súa táboa mostra a lista desa
pista e desprázase para levar a súa fila ao principio, salvo que usaras a
táboa nos últimos 10 segundos (desprazala, arrastrar unha pista, abrir o
menú dunha pista ou facer clic nunha lapela): entón agarda a que a deixes en
paz ese tempo. O tempo é `ui.follow_current_grace_secs` en `config.json`; 0
desactiva o seguimento.

Os reprodutores son independentes: varios reprodutores poden mostrar a mesma
lista, cada un coa súa propia pista seguinte, as súas propias marcas de
reproducida e os seus propios tempos no pé. Reproducir, parar ou saltar nun
reprodutor nunca move a pista seguinte doutro. A mesma pista pode incluso
estar en antena en dous reprodutores á vez. Editar a lista (engadir, mover ou
quitar entradas) ou que un ficheiro pase a ser ilexible aínda pode cambiar a
seguinte de calquera reprodutor que a mostre.

Cores das filas:

| Fila | Significado |
|---|---|
| **Vermella**, cunha icona de altofalante (ou pausa) | En antena neste reprodutor. Tamén pode mostrar a frecha verde: a pista en antena é tamén a seguinte, así que se reproduce unha vez máis |
| **P2** vermello (ou outro número) na columna de número | En antena nese reprodutor |
| **Verde**, cunha frecha | A pista seguinte deste reprodutor |
| Atenuada | Xa reproducida neste reprodutor |
| Ficheiro cunha icona de cruz / aviso | Ficheiro que falta / ilexible (sáltase); pon o punteiro sobre a fila: a xanela emerxente comeza coa razón, en ámbar, e despois os campos habituais. Un ficheiro que falta búscase de novo cada 30 s (`tuning.missing_recheck_ms`). |
| Frechas de recarga á dereita do título | Analizada por unha versión anterior; aínda se reproduce con esa análise. **Configuración → Análise → Analizar as pistas desactualizadas** póñea ao día (as pistas dun reprodutor actualízanse igualmente) |
| Reloxo de area á dereita do título | A pista está á espera da súa análise (pon o punteiro sobre o reloxo de area: *Análise pendente*). Reprodúcese igualmente, e o reloxo de area desaparece cando remata a análise |
| Violeta | Seleccionada |

**Dica da pista.** Pon o punteiro un momento sobre unha fila para ver o seu título, artista, álbum, data, xénero, duración, formato (tipo, frecuencia de mostraxe e profundidade de bits cando se coñecen) e a ruta do seu ficheiro. Un campo que o ficheiro non ten omítese. A xanela emerxente é a única información ao pasar o punteiro sobre unha fila, e nunca se move mentres se mostra. Para un ficheiro que falta ou é ilexible, comeza coa razón.

## Rato {#mouse}

- **Clic** selecciona unha pista. **Dobre clic** faina a pista seguinte deste
  reprodutor. Na pista en antena, fai que se reproduza unha vez máis cando
  remate a pasada actual.
- **Clic dereito** abre o menú contextual:

![O menú contextual dunha pista](../../images/guide/track-menu.png)

| Elemento | Acción |
|---|---|
| Reproducir agora | Inicia esta pista ao instante (mesturando se o reprodutor está en antena) |
| Marcar como seguinte | O mesmo que o dobre clic. Na pista en antena, reprodúcese unha vez máis, desde o seu comezo, cando remate a pasada actual (mesturando como Repetir, sen pausa), e despois o reprodutor segue. Actúa unha soa vez. Stop ao final, o modo SINGLE e unha marca de parar despois seguen detendo antes o reprodutor. Mentres hai un CUE en marcha, móvese á nova pista seguinte |
| Preescoitar no CUE | Reprodúceo na saída CUE (abre a xanela CUE). Atenuado cando o reprodutor non ten saída Cue á parte da súa Main |
| Editar etiquetas… | Abre o editor de etiquetas desta pista. **Gardar** escribe os cambios no ficheiro de audio; **Cancelar** (ou Esc, cando non hai un gardado en curso) pecha sen escribir. O elemento aparece atenuado, coa razón ao poñer o punteiro enriba, mentres a pista está en antena, en CUE ou nun cartucho que soa, mentres aínda non se leron as súas etiquetas, cando o ficheiro falta, e para os formatos cuxas etiquetas non se poden escribir (por exemplo DSD) |
| Volver analizar | Analiza esta pista de novo agora, sexa cal for o seu estado. Un ficheiro ilexible que se arranxou tamén se recolle por si só (consulta [Resolución de problemas](troubleshooting.md)). Os marcadores manuais consérvanse |
| Engadir pistas debaixo… | Escolle ficheiros para inserir despois desta pista |
| Duplicar | Insire unha copia sen reproducir debaixo (coas súas marcas de repetir e parar despois) |
| Repetir esta pista | Marca para reproducila unha e outra vez, sen pausa, ata que premas Play (seguinte), Anterior, Stop ou Stop con fundido, ou actives Stop ao final. Pausa mantena repetindo. Unha icona de repetición móstrase antes do título |
| Parar despois desta pista | Marca para deter o reprodutor cando remate esta pista, cada vez que se reproduza (en calquera modo). A diferenza do botón **Stop ao final** do reprodutor, a marca queda coa pista e gárdase coa lista. A icona de parar despois móstrase antes do título. Gaña a Repetir |
| Mover a ▸ | Móveo ao final doutra lista |
| Quitar da lista | Quítao; non é posible mentres está en antena |

## Editar etiquetas {#editing-tags}

**Editar etiquetas…** abre unha xanela para unha pista. Mentres está aberta
non actúa ningún atallo de teclado, e os ficheiros soltados na xanela da
aplicación ignóranse.

![O editor de etiquetas dun ficheiro FLAC, coa súa portada, título, artista, álbum, data e xénero](../../images/guide/tag-editor.png)

- **O que ves.** O editor le o ficheiro ao abrirse (mentres tanto mostra
  «Lendo as etiquetas…»). Sempre se mostran: título, artista, álbum, artista
  do álbum, data, número de pista e total, número de disco e total, xénero,
  compositor e comentario. Móstranse cando o ficheiro os ten: subtítulo,
  agrupación, BPM, tonalidade inicial, estado de ánimo, ISRC, editorial,
  número de catálogo, copyright, artista orixinal, álbum orixinal, data de
  lanzamento orixinal, letrista, director, remesturador, arranxista,
  intérprete, idioma, codificado por, letra, ordenar por título, ordenar por
  artista, ordenar por álbum, ordenar por artista do álbum, ordenar por
  compositor e web do artista.
- **Engadir campo.** O menú debaixo dos campos lista os demais campos. Ofrece
  só o que o formato de etiquetas do ficheiro pode gardar (un WAV con RIFF
  INFO, un AIFF ou unha etiqueta ID3v1 antiga gardan menos campos que ID3v2,
  FLAC ou MP4), e aparece atenuado cando non queda nada por engadir. Un dos
  campos sempre mostrados que o formato non pode gardar aparece en gris cunha
  nota. Baleirar un campo quítao do ficheiro; un campo engadido e deixado
  baleiro non se escribe.
- **Varios valores.** Os campos que poden conter varios valores (artista,
  artista do álbum, xénero, compositor, estado de ánimo e os créditos como
  letrista, director, remesturador, arranxista e intérprete, e idioma)
  mostran un valor por liña; **Gardar** escribe un valor por liña á maneira
  propia do formato. Comentario e letra son texto libre en varias liñas.
- **Comprobacións.** Data e data de lanzamento orixinal seguen ISO 8601
  (`2019`, `2019-05` ou `2019-05-14`, opcionalmente cunha hora); o número de
  pista e de disco, os seus totais e o BPM son números enteiros, e un total
  necesita o seu número. Un campo cun valor non válido queda marcado e
  **Gardar** segue desactivado. Un valor que o ficheiro xa tiña e que non
  tocaches consérvase tal como está.
- **Campos demasiado longos.** Un campo cuxo texto é máis longo que
  `limits.max_tag_chars`, ou que ten máis valores que `limits.max_tag_values`,
  móstrase só de lectura coa nota «Demasiado longo para editalo aquí;
  consérvase tal como está no ficheiro». Nunca se volve escribir, así que un
  gardado non o pode cortar.
- **O que se conserva.** Todo o que o editor non mostra (outras claves
  estándar, claves personalizadas, imaxes distintas da portada frontal,
  cadros binarios) queda no ficheiro cos mesmos valores. O editor indica
  cantas etiquetas se conservan así (e «outras» cando o formato ten cadros
  que non se poden contar). Gardar recodifica os elementos que o editor
  asigna, así que un elemento conservado pode diferir nos seus bytes
  (codificación do texto, orde dos cadros) pero non no seu valor.
- **A portada.** O editor mostra a portada frontal, ou a primeira imaxe do
  ficheiro cando non hai portada frontal, como miniatura.
  - **Cambiar…** abre un diálogo de ficheiros para unha imaxe JPEG ou PNG (como
    máximo `limits.max_cover_bytes`, e debe poder decodificarse). Se non, o
    editor di por que e non cambia nada.
  - **Quitar** limpa a portada frontal. Está desactivado cando o ficheiro non
    ten portada frontal: unha imaxe que se mostra só porque non hai portada
    frontal é só para ver e consérvase tal como está.
  - Unha portada que está no ficheiro pero non se pode mostrar (unha imaxe que
    non se decodifica, ou un GIF, BMP ou WebP) anúnciase con «Esta portada
    non se pode mostrar; consérvase tal como está». **Cambiar…** e
    **Quitar** seguen funcionando.
  - O cambio escríbese con **Gardar** e descártase con **Cancelar**. As
    contraportadas e todas as demais imaxes nunca se tocan. Un formato sen
    lugar para imaxes (WAV con RIFF INFO, AIFF, ID3v1) mostra a área
    desactivada. Despois de gardar, a portada do reprodutor mostra a nova.
- **Como funciona un gardado.** O ficheiro cópiase ao carón do orixinal, a
  copia recibe as etiquetas, sincronízase e substitúe o orixinal, así que un
  fallo deixa o ficheiro como estaba. A razón móstrase no editor, que segue
  aberto para tentalo de novo, e na barra de estado. Só se escriben os campos
  que cambiaches. Despois de gardar, a táboa mostra as novas etiquetas ao
  instante, e os marcadores e a forma de onda consérvanse. Se o ficheiro non
  conservou un campo que cambiaches, a barra de estado nomeao.
- **Despois dunha actualización.** As pistas dunha versión anterior reciben
  en segundo plano, sen facer ruído, a súa data, xénero e outras etiquetas
  (sen unha análise completa).

## Arrastrar e soltar {#drag-and-drop}

- Arrastra unha pista dentro da lista para reordenala. Unha liña violeta
  mostra onde caerá: está no límite de fila máis próximo ao punteiro, e só na
  lista que hai baixo o punteiro. Soltar sobre a cabeceira, o bordo dunha
  columna, a barra de desprazamento ou unha xanela que cubre a lista (a xanela
  CUE) non solta nada.
- Mentres arrastras unha pista, mantén o punteiro preto do bordo superior ou
  inferior dunha lista para desprazala: canto máis preto do bordo, máis rápido
  vai, e detense nos extremos da lista ou cando te afastas do bordo. A roda do
  rato tamén despraza a lista durante o arrastre. A liña violeta segue o
  punteiro mentres a lista se move. Arrastrar ficheiros desde o xestor de
  ficheiros sobre unha lista desprázaa do mesmo xeito onde o sistema informa
  da posición do punteiro, unha vez que moves o punteiro sobre a lista.
- Arrástraa á lista doutro reprodutor para movela alí.
- Arrástraa a unha lapela para engadila ao final desa lista.
- Solta ficheiros ou cartafoles desde o xestor de ficheiros sobre unha lista
  para inserilos na posición onde soltas. Sobre a cabeceira, o bordo dunha
  columna, a barra de desprazamento ou unha xanela que cubre a lista non se
  insire nada. Se o sistema non informa da posición, van ao final da lista
  que se ve.

## Pé {#footer}

**+ Engadir** abre un diálogo de ficheiros, que comeza no cartafol de música
definido en Configuración. **Reiniciar** (a icona de frecha ao seu carón)
quita a marca atenuada de «xa reproducida» de todas as pistas da lista, para
todos os reprodutores, despois de preguntar «Quitar a marca de reproducida de
todas as pistas desta lista?» (**Cancelar**, Esc ou un clic fóra conservan as
marcas). A pista que está en antena conserva o seu estado e márcase cando o
reprodutor a deixa. O botón aparece atenuado cando non hai nada que quitar. O
pé tamén mostra o número de pistas, o tempo que queda na lista e a súa
duración total.

## Ficheiros de lista {#playlist-files}

- **Importar:** Configuración → Listas → **Importar M3U / PLS…**, ou solta un
  ficheiro `.m3u`, `.m3u8` ou `.pls` na xanela. Convértese nunha lista nova
  co nome do ficheiro.
  - As rutas relativas resólvense a partir do cartafol do ficheiro de lista.
  - Enténdense os enderezos `file://`.
  - Os ficheiros que non se atopan engádense igualmente, marcados como non
    dispoñibles.
  - Os streams de Internet omítense; unha mensaxe di cantos.
- **Exportar:** o botón **M3U** de cada lista en Configuración gárdaa como un
  ficheiro M3U8 con títulos, duracións e rutas completas.
