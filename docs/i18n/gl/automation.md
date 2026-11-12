# Automatización

A automatización mantén a emisora en antena cando non hai ninguén no estudo.
Esta versión ten o **musicalizador**, un reprodutor que escolle a música por
si só, a partir das regras que fixas, e que nunca se detén por falta de
lista, e a columna **Grella**, que executa os bloques horarios do día, co
editor onde os creas.
Os reprodutores que xa usas (P1 … Pn) non se tocan: seguen sendo teus para
os programas que preparas e emites en directo.

## A columna Grella {#the-schedule-column}

A columna **Grella** reproduce os **bloques** do día: grupos de elementos cunha
hora de inicio (un indicativo horario, cuñas, un programa). Móstraa coa tesela
**Grella** da barra superior, igual que mostras un reprodutor.

![A columna Grella en marcha, xunto á columna Música: os bloques do día por hora de inicio, cada un co seu indicativo da emisora (o reloxo márcao como puntual), cuñas e a rotación que o musicalizador reproduce despois; a canción do musicalizador en antena e as seguintes](../../images/guide/automation-columns.png)

A súa lista é a grella, agrupada por bloques: a hora de inicio de cada bloque
e despois os seus elementos. Mostra hoxe, o último bloque de onte e o primeiro
de mañá. Un bloque cambiado para un día indícao. O elemento en antena e o
seguinte márcanse como nun reprodutor; os elementos que xa se emitiron
aparecen atenuados, e un elemento que non se emitiu (**Non emitido**) aparece
riscado. Un reloxo marca un elemento **puntual**, que só soa á súa hora.

Enriba da lista:

- **Iniciar** pon en marcha a grella: cada bloque soa á súa hora e o
  musicalizador soa entre bloques. Nada se pon en marcha por si só: despois
  de cada arranque da aplicación, a automatización está detida ata que premes
  Iniciar.
- **Deter tras a actual** deixa que remate o que está en antena, e despois xa
  non soa nada por si só.
- **En marcha** ou **Detida** di en que estado está.

Para cambiar o que soa a continuación, fai dobre clic nun elemento, ou fai
clic dereito nel e escolle **Que soe a continuación**; segue á canción que
está en antena. **Volver ao reloxo**, no mesmo menú, deixa que a grella volva
escoller segundo a hora.

## Editar a grella {#editing-the-schedule}

**Editar grella**, na columna Grella, abre o editor. Ten tres lapelas, e
**Rotacións…** lévate a Configuración → Automatización, onde se crean as
rotacións.

![O editor da grella, lapela Modelos: os modelos Base e Weekend á esquerda; os bloques de Base á dereita, cada un co seu inicio, os seus elementos, Engadir de categoría e a rotación da música que vai despois](../../images/guide/schedule-templates.png)

- **Modelos.** Un modelo é un día de bloques; unha instalación nova ten un,
  "Base", para todos os días. Crea, duplica, renomea ou elimina modelos á
  esquerda; os bloques do escollido están á dereita. Escribe o inicio dun
  bloque como `HH:MM:SS` (unha hora que non existe queda no campo cunha nota,
  e non cambia nada). Arrastra pistas da biblioteca a un bloque para
  engadilas; **Engadir de categoría** engade un elemento que reproduce unha
  pista desa categoría, unha distinta cada vez; **Música despois** escolle a
  rotación que o musicalizador reproduce despois do bloque (a rotación
  predeterminada se non se escolle ningunha). Cada elemento pódese marcar como
  **Puntual**, mover ou quitar. Cando os elementos dun bloque duran máis que o
  tempo ata o seguinte bloque, o bloque di canto, en ámbar; aínda así podes
  mantelo.
- **Semana tipo.** Que modelo usa cada día da semana.

  ![A lapela Semana tipo: Base de luns a venres, Weekend o sábado e o domingo](../../images/guide/schedule-week.png)

- **Días.** Escolle unha data no calendario (as datas con cambios aparecen en
  ámbar). Unha data pode usar outro modelo que o do seu día da semana (un
  festivo nun xoves pode usar o do fin de semana). Os seus bloques pódense
  cambiar só para esa data: o cambio márcase, e **Volver ao modelo** desfaino.
  **Este día non** quita un bloque só desa data, e **Engadir bloque este día**
  engade un. Os cambios posteriores do modelo seguen chegando aos bloques que
  non cambiaches nesa data.

  ![A lapela Días: o calendario co día de hoxe escollido e as datas con cambios en ámbar; o primeiro bloque de hoxe ten unha cuña máis, marcada "cambiado para este día", con Volver ao modelo](../../images/guide/schedule-days.png)

## O musicalizador {#the-music-filler}

O musicalizador é a columna **Música**. Está oculta ata que a mostras: fai
clic na tesela **Música** da esquerda da barra superior. Fica á esquerda de
P1.

Ten o aspecto e funciona como un reprodutor: o reloxo grande, a forma de onda,
os botóns de transporte, o fader e a lista de pistas. A diferenza é a lista:

- **énchese por si soa** a partir dunha **rotación** (abaixo), sempre con
  preto de media hora por diante (*marxe por diante*, en Configuración →
  Automatización);
- **recórtase por si soa**: conserva as últimas 50 pistas que reproduciu e
  quita as máis antigas, así que pode funcionar durante semanas;
- antes dun bloque da grella pode **cambiar as cancións que escolleu por si
  só** por outras cuxa duración se axuste ao tempo que queda (consulta
  abaixo); as cancións que engadiches nunca se tocan;
- aínda podes **cambiala a man**: arrastra pistas da biblioteca, reordena
  filas, quita unha canción que non queres. Unha fila que engades márcase cunha
  man e nunca se substitúe; unha fila que quitas non conta como reproducida.

As súas saídas fíxanse en Configuración → Saídas de audio, na fila **Música**,
como para calquera reprodutor. Non ten número, nin atallos de teclado nin
asignacións MIDI, e o control remoto non a ve.

Preme **Play** para iniciala e **Deter tras a actual** para deixar que remate
a canción que está en antena e parar. Como todos os reprodutores, nunca
arranca por si soa, nin sequera despois de reiniciar.

Enriba da súa lista, onde un reprodutor ten as lapelas das listas, a columna
Música mostra a **rotación** que reproduce. Escolle outra aló: as cancións que
tiña na cola substitúense, as que engadiches quedan. Cando a rotación non pode
atopar nada que reproducir (as súas categorías están baleiras, ou todos os
ficheiros faltan), a barra dío.

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

Con un rango, un indicativo da emisora vén despois de dúas cancións unha vez e
despois de tres a seguinte. Un rango que empeza en **0** deixa que un paso se
salte algúns turnos: con Clásicos en 0–1, unha canción clásica soa despois do
indicativo da emisora algunhas veces e outras non. O máximo é sempre polo
menos 1. Unha rotación é a **predeterminada**: unha instalación nova ten unha,
**Music**, que reproduce toda a túa música.

As rotacións créanse en Configuración → Automatización:

![Configuración, Automatización: as rotacións Music (a predeterminada) e Hits, a Music seleccionada co seu único paso sobre toda a música, os seus contadores Como mínimo e Como máximo, e as regras do musicalizador](../../images/guide/settings-automation.png)

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
esvaece cando comeza o bloque perde como máximo 30 segundos, e non comeza
ningunha canción que soaría menos de 45 segundos. Cando ningunha canción da
rotación pode cumprir as dúas cousas, soa a que menos perde. Os dous valores
só se poden cambiar por agora no ficheiro de configuración
(`automation.max_fade_cut_secs` e `automation.min_song_play_secs`).

Un **Indicativo horario** é un tipo de pista propio (Biblioteca → Tipo ▸
Indicativo horario), para a identificación da emisora ao comezo da hora.
