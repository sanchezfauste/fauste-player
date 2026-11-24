# Automatización

A automatización mantén a emisora en antena cando non hai ninguén no estudo.
Esta versión ten o **musicalizador**, un reprodutor que escolle a música por
si só, a partir das regras que fixas, e que nunca se detén por falta de
lista, e a columna **Grella**, que executa os bloques horarios do día, feitos
a partir de modelos en Configuración.
Os reprodutores que xa usas (P1 … Pn) non se tocan: seguen sendo teus para
os programas que preparas e emites en directo.

## A columna Grella {#the-log-column}

A columna **Grella** reproduce os **bloques** do día: grupos de elementos cunha
hora de inicio (un indicativo horario, cuñas, un programa). Móstraa coa tesela
**Grella** da barra superior, igual que mostras un reprodutor.

![A columna Grella en marcha, xunto á columna Musicalizador: os bloques do día por hora de inicio, cada un co seu indicativo da emisora (o reloxo márcao como de hora fixa), cuñas e a rotación que o musicalizador reproduce despois; a canción do musicalizador en antena e as seguintes](../../images/guide/automation-columns.png)

A súa lista parece unha lista de reprodución, agrupada por bloques. Cada
bloque comeza cunha fila de cabeceira: a súa hora de inicio, cantos elementos
ten e canto duran, "cambiado para este día" cando o cambiaches hoxe e, en
ámbar, canto se pasa do bloque seguinte. Despois veñen os seus elementos, coa
**Hora** á que está previsto que comece cada un. A lista mostra hoxe, o último
bloque de onte e o primeiro de mañá. O elemento en antena e o seguinte
márcanse como nun reprodutor; os elementos que xa se emitiron aparecen
atenuados, e un elemento que non se emitiu (**non emitido**) aparece riscado.
Un reloxo marca un elemento de **hora fixa**, que só soa á súa hora. Unha
**pista de categoría** mostra a pista escollida para ela en canto pasa a ser a
seguinte (ou vén despois da seguinte no seu bloque), coa categoría detrás:
esa é a pista que soa. Se a categoría non ten ningunha pista que poida soar
(non ten ningunha asignada, ou faltan os seus ficheiros), a fila dío en
ámbar, e o elemento non se emitirá. Un elemento cuxo ficheiro resulta
ilexible cando vai soar sáltase e tamén se marca como non emitido. As pistas
asígnanse a unha categoría na biblioteca. A fila do **musicalizador** di ata
cando soa, e móstrase en antena mentres o musicalizador soa para a Grella. Un
seguinte cuxa hora xa pasou mostra a hora en ámbar. Arrastra a liña entre
dúas cabeceiras de columna para cambiar o ancho das columnas, como nunha lista
de reprodución; as xanelas de modelo e de día usan os mesmos anchos.

### Cambiar a grella de hoxe {#changing-todays-log}

A Grella cámbiase como unha lista de reprodución, e o cambio é só para ese
día: o modelo queda como está.

- Os botóns da parte inferior da columna: **+ Pista…** abre a biblioteca,
  para arrastrar pistas á lista; **+ Pista de categoría…** e
  **+ Musicalizador (rotación)…** preguntan cal e engádena xusto debaixo da
  fila que seleccionaches (fai clic nunha fila para seleccionala; selecciona
  unha cabeceira de bloque para engadir ao principio dese bloque), ou, sen
  nada seleccionado ou debaixo dunha fila que xa non pode cambiar, ao final do
  próximo bloque; **+ Bloque horario…** pide unha hora, coa primeira libre xa
  posta. Cando a columna é demasiado estreita para todos, os que non caben
  están detrás de **⋯**.
- Arrastra pistas da biblioteca, ou ficheiros de audio do teu xestor de
  ficheiros, a unha fila para poñelos diante dela (ou detrás, na metade
  inferior da fila, como nunha lista de reprodución: así van ao final dun
  bloque), e arrastra filas dentro do mesmo día para movelas. A fila que
  engades queda seleccionada e á vista, así que a seguinte que engadas vai
  debaixo.
- Fai clic dereito nunha fila para **Hora fixa** e **Quitar da grella de
  hoxe**. Fai clic (ou clic dereito) nunha cabeceira de bloque para engadir
  unha **Pista…**, unha **Pista de categoría…** ou un **Musicalizador
  (rotación)…** ao final dese bloque, ou para **Cambiar a hora…**, **Quitar
  bloque** e, nun bloque cambiado, **Volver ao modelo**.
- Mentres arrastras sobre unha cabeceira de bloque, a liña de debaixo indica
  que o soltas ao principio dese bloque; para poñer filas ao final do bloque
  de enriba, sóltaas na metade inferior da súa última fila.
- O que xa soou, o elemento en antena e os días pasados non se poden cambiar.
  Un cambio no bloque en antena ten efecto despois do elemento en antena.

A columna Grella ten a cabeceira e os botóns dun reprodutor, e funcionan como
nun reprodutor:

- **Reproducir** reproduce o elemento marcado como seguinte e pon en marcha a
  automatización: cada bloque soa entón á súa hora, e o musicalizador soa
  entre bloques. Cando o seguinte o escolleu o reloxo e o seu bloque comeza
  máis tarde, a música soa ata entón. Nada se pon en marcha por si só: despois
  de cada arranque da aplicación, a automatización está detida ata que premes
  Reproducir.
- **Pausa**, **Stop** e **Stop con fundido** actúan sobre o que está en antena
  para a Grella (o seu elemento, ou a canción do musicalizador entre bloques)
  e deteñen a automatización. Reproducir despois de Pausa segue desde onde
  estaba; mentres está en pausa, o seguinte non se move, e ao retomar o bloque
  seguinte mantén a súa hora. Reproducir despois dun Stop durante a música
  volve poñer música, e o bloque seguinte comeza á súa hora.
- **Stop ao final** deixa que remate o que está en antena, e despois a
  automatización detense. Se está a soar a música e antes chega a hora dun
  bloque, o bloque comeza puntual e a automatización detense despois do seu
  primeiro elemento.
- **Pista anterior** non fai nada na Grella.
- **SINGLE** fai manual a Grella: Reproducir reproduce o elemento seguinte, e
  detense despois del; ningún bloque comeza por si só.

Mentres o musicalizador soa entre bloques, a cabeceira da Grella di
**Musicalizador · *rotación*** (a rotación que reproduce) e os seus botóns
seguen funcionando; a canción, o seu medidor e a súa forma de onda están só
na columna Musicalizador.

Cando a automatización se detén despois dun elemento, o seguinte queda no
elemento de debaixo do que soou, e Reproducir reprodúceo, sexa a hora que
sexa: escolles ti. Cando se detén durante a música, Reproducir volve poñer
música ata o bloque seguinte, salvo que escollas outro seguinte. Para cambiar
o que soa a continuación, fai dobre clic nun elemento, ou fai clic dereito
nel e escolle **Que soe a continuación**. **Volver ao reloxo**, no mesmo
menú, pon como seguinte o que di a hora.

## Modelos e días {#templates-and-days}

A Grella reproduce cada día a partir dun **modelo**: un día de bloques. Unha
instalación nova ten un, "Base", para todos os días. Os modelos fanse en
Configuración → **Automatización**, en **Modelos e días** (non en Android); a
columna Grella só cambia o día de hoxe, como se viu arriba.

![Configuración, Automatización, Modelos e días: os modelos Base e Weekend con Editar…, Novo modelo, Duplicar, Eliminar e o nome; a semana tipo, Base de luns a venres e Weekend o sábado e o domingo; o mes con hoxe escollido, as datas con cambios en ámbar, o modelo da data e Editar día…](../../images/guide/settings-plans.png)

- **Modelos.** Escolle un e preme **Editar…** (ou fai dobre clic nel) para
  abrilo; **Novo modelo**, **Duplicar** e **Eliminar** están ao lado, e o seu
  nome debaixo.
- **Semana tipo.** Que modelo usa cada día da semana.
- **Días.** Escolle unha data no calendario (as datas con cambios están en
  ámbar). Unha data pode usar outro modelo distinto do do seu día da semana
  (un festivo en xoves pode usar o da fin de semana). **Editar día…** abre
  esa data, para cambiar os seus bloques só para esa data; as datas pasadas
  non se poden cambiar.

Un modelo ou unha data ábrese na súa propia xanela, que pecha Configuración.
É a mesma lista que a columna Grella, e cámbiase igual: os botóns da súa parte
inferior (Pista…, Pista de categoría…, Musicalizador (rotación)…, Bloque
horario…, que engaden debaixo da fila que seleccionas), pistas arrastradas
desde a biblioteca (ou ficheiros desde o teu xestor de ficheiros) a unha fila,
filas arrastradas para movelas, o menú de clic dereito das filas e o menú das
cabeceiras de bloque. Nunca soa, e pode quedar aberta xunto á biblioteca. A
xanela de hoxe mostra o que soou e o que soa, como a Grella; a xanela dun día
péchase á medianoite, cando ese día xa non pode cambiar.

![Un modelo aberto na súa xanela: os bloques do modelo Base por hora de inicio, cada un co seu indicativo da emisora, as súas cuñas e o musicalizador despois, e Engadir na parte inferior](../../images/guide/template-popup.png)

Unha **pista de categoría** nun modelo reproduce unha pista desa categoría,
unha distinta cada vez. O elemento **Musicalizador** escolle a rotación que o
musicalizador reproduce despois do bloque (a rotación predeterminada cando o
bloque non ten ningunha). Cando os elementos dun bloque duran máis que o tempo
ata o bloque seguinte, a súa cabeceira di en canto, en ámbar; aínda así podes
deixalo así.

Un cambio nunha data queda marcado como "cambiado para este día" no seu
bloque, e **Volver ao modelo**, no menú da cabeceira do bloque, desfaino. Os
cambios posteriores no modelo seguen chegando aos bloques que non cambiaches
nesa data. Un cambio nun modelo feito mentres un dos seus bloques está en
antena deixa ese bloque como está para hoxe (mostra "cambiado para este
día"); o cambio chégalle a partir do día seguinte.

## O musicalizador {#the-music-fill}

O musicalizador é a columna **Musicalizador**. Está oculta ata que a mostras:
fai clic na tesela **Musicalizador** da esquerda da barra superior. Fica á
esquerda de P1.

Ten o aspecto dun reprodutor e funciona como tal: o reloxo grande, a forma de
onda, os botóns de transporte, o fader e a lista de pistas. A diferenza está
na lista:

- **énchese soa** a partir dunha **rotación** (máis abaixo), sempre cunha
  media hora de adianto (a *marxe por diante*, en Configuración →
  Automatización);
- **recórtase soa**: conserva as últimas 50 pistas que reproduciu e quita as
  máis antigas, de modo que pode funcionar durante semanas;
- antes dun bloque da grella pode **cambiar cancións que escolleu ela mesma**
  por outras cuxa duración encaixe no tempo que queda (véxase máis abaixo); as
  cancións que engadiches ti nunca se tocan;
- aínda podes **cambiala a man**: arrastrar pistas desde a biblioteca,
  reordenar filas, quitar unha canción que non queres. Unha fila que engades
  márcase cunha man e nunca se substitúe; unha fila que quitas non conta como
  reproducida.

As súas saídas axústanse en Configuración → Saídas de audio, na fila
**Musicalizador**, como para calquera reprodutor. Non ten número, nin atallos
de teclado nin asignacións MIDI, e o control remoto non a ve.

Preme **Reproducir** para poñela en marcha e **Stop ao final** para deixar que
remate a canción que está en antena e se deteña. Como todos os reprodutores,
nunca se pon en marcha por si soa, nin sequera despois dun reinicio. Mentres
soa para a Grella entre bloques, detela ou poñela en pausa tamén detén a
automatización, e o seu Stop ao final detén a automatización cando remata a
canción; se antes chega a hora dun bloque, o bloque soa e a marca quítase.
Despois de poñer en pausa a Grella durante a música, Reproducir na columna
Musicalizador reproduce a canción pola súa conta: o Reproducir da Grella xa
non a retoma, e o seu Stop xa non a detén.

Enriba da súa lista, onde un reprodutor ten as lapelas das listas, a columna
Musicalizador mostra a **rotación** que reproduce. Escolle outra aló: as
cancións que tiña na cola substitúense, as que engadiches quedan. Cando a
rotación non pode atopar nada que reproducir (as súas categorías están
baleiras, ou todos os ficheiros faltan), a barra dío.

## Rotacións {#rotations}

Unha **rotación** é unha lista curta de **pasos** que o musicalizador reproduce
por quendas, e despois de novo desde o primeiro. Cada paso nomea de onde saen
as súas pistas (unha categoría da biblioteca coas súas subcategorías, ou todas
as pistas dun tipo) e cantas reproduce cada vez, como un rango:

| Paso | Reproduce |
|---|---|
| Éxitos actuais | 2–3 |
| Indicativos da emisora | 1 |
| Clásicos | 0–1 |

Cun rango, un indicativo da emisora chega despois de dúas cancións unha vez e
despois de tres a seguinte. Un rango desde **0** permite que un paso salte
algunhas quendas: con Clásicos en 0–1, unha canción clásica soa despois do
indicativo unhas veces e outras non. O máximo sempre é polo menos 1. Unha
rotación é a **predeterminada**: unha instalación nova ten unha, **Música**,
que reproduce toda a túa música.

Un paso cuxa categoría non ten ningunha pista que poida soar (unha categoría
baleira, ou ficheiros que faltan) reproduce na súa quenda unha canción da
rotación predeterminada (calquera música, se o paso é da propia rotación
predeterminada), de modo que os demais pasos manteñen o seu lugar: un
indicativo da emisora segue chegando entre cancións.

As rotacións créanse en Configuración → Automatización:

![Configuración, Automatización: as rotacións Music (a predeterminada) e Hits, a Music seleccionada co seu único paso sobre toda a música, as súas cantidades Como mínimo e Como máximo, e o comezo de Modelos e días](../../images/guide/settings-automation.png)

## Como escolle {#how-it-chooses}

Dentro dun paso, o musicalizador prefire as pistas que levan máis tempo sen
emitirse, cunha pequena marxe de azar entre elas para que a orde nunca se
converta nun ciclo fixo. Tamén mantén separado:

- **a mesma pista:** pode volver soar cando pasasen 3 horas ou cando soase a
  metade das demais pistas do seu paso desde entón, o que ocorra primeiro;
- **o mesmo artista:** non dentro de 30 minutos.

Conta todo o que saíu en antena, do musicalizador, dun reprodutor ou da
cartucheira; a preescoita non. Cando unha categoría pequena non pode manter as
regras, o musicalizador cede en lugar de quedar en silencio: primeiro no
artista, despois na pista. Configuración → Automatización ten todos os valores,
e é onde fas, renomeas e cambias rotacións (consulta
[Configuración](settings.md#automation)).

Antes dun bloque da grella, o musicalizador tamén escolle as súas últimas
cancións pola súa duración, para que a música remate limpa: a canción que se
esvaece cando comeza o bloque segue a soar á hora do bloque e perde como
máximo 30 segundos despois dela, e non comeza ningunha canción que soaría menos
de 45 segundos. Tamén mantén as regras de artista e de pista sempre que pode.
Cando ningunha canción da rotación pode cumprir as dúas cousas, soa a que menos
perde, así que ás veces poden oírse uns segundos dunha canción. Unha pausa da
música move as cancións que van detrás, e o musicalizador volve escollelas. O
musicalizador tamén evita as cancións, e os artistas, que un bloque reproduce
pouco despois. Os dous valores só se poden cambiar por agora no ficheiro de
configuración (`automation.max_fade_cut_secs` e
`automation.min_song_play_secs`).

Un **Indicativo horario** é un tipo de pista propio (Biblioteca → Tipo ▸
Indicativo horario), para a identificación da emisora ao comezo da hora.
