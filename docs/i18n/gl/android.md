# Android

Fauste Player tamén funciona en teléfonos e tablets Android (Android 10 ou
posterior), sobre o mesmo núcleo de emisión ca o escritorio: os reprodutores,
as listas, a cartucheira, os marcadores, a normalización de sonoridade e
todas as regras de reprodución funcionan do mesmo xeito.

## Instalación {#install}

Unha versión que inclúe a aplicación ten `fauste-player-<version>-android.apk`
na súa páxina de **Releases** (co seu `.sha256`). Ábrea no teléfono e permite
que o teu navegador ou xestor de ficheiros instale aplicacións cando Android
llo pida. Para actualizar, instala o APK máis novo do mesmo xeito: as túas
listas e a túa configuración quedan. A aplicación aínda non está en Google
Play.

A versión 0.13.0 e as anteriores instálanse como outra aplicación, así que un
APK máis novo non as actualiza: aparece como un segundo Fauste Player.
Desinstala a antiga cando a nova estea configurada. As súas listas e a súa
configuración non se trasladan.

## Primeiro arranque {#first-start}

- **Acceso á música.** Permite o acceso á música (e ás notificacións, en
  Android 13 e posterior). Sen el, o navegador de música queda baleiro.
- **A emisión continúa.** Mentres soa un reprodutor ou un cartucho, queda
  unha notificación e o audio segue coa pantalla apagada ou coa aplicación en
  segundo plano. Como no escritorio, nada sae en antena por si só despois dun
  inicio ou dun fallo.
- **Atrás e pechar.** Atrás pecha primeiro o que estea aberto (o navegador de
  música, Configuración, Acerca de, un reprodutor aberto, unha páxina da
  lapela Biblioteca). Se non queda nada por pechar, Atrás garda a aplicación
  mentres algo soa (un reprodutor, un cartucho ou unha preescoita CUE) e segue
  reproducindo; cando nada soa, garda e pecha a aplicación. Deslizar a
  aplicación fóra do selector de aplicacións fai o mesmo: deslizar nunca
  detén o que se está reproducindo, e a aplicación péchase cando queda en
  silencio.
- **O son doutras aplicacións.** Cando outra aplicación toma o audio, Fauste
  Player segue reproducindo e dío na barra de estado da parte de abaixo.

## A pantalla {#the-screen}

Nun **teléfono** a barra superior ten catro lapelas, **Directo**,
**Cartucheira**, **Listas** e **Biblioteca**, mostradas como iconas (a
escollida tamén co seu nome cando hai sitio), e despois o **cadeado** e **⋮**
(Máis: Configuración e Acerca de). O reloxo móstrase cando hai sitio para el.
Co teléfono de lado, a pantalla Listas pon os seus chips de reprodutor (P1,
P2…) na barra superior, e a barra de estado da parte de abaixo só se mostra
cando ten algo que dicir.

<p>
<img src="../../images/android/1-live.png" width="180" alt="Directo nun teléfono: dous reprodutores como tarxetas, unha enteira e o comezo da seguinte, e a cartucheira debaixo">
<img src="../../images/android/2-carts.png" width="180" alt="Cartucheira nun teléfono: a páxina General en vertical, dous cartuchos por fila">
<img src="../../images/android/3-playlists.png" width="180" alt="Listas nun teléfono: os chips de P1 a P4, as lapelas das listas, a lista de pistas e a tarxeta do reprodutor na parte de abaixo">
<img src="../../images/android/4-music.png" width="180" alt="Música do dispositivo nun teléfono: o cartafol Demo con tres cancións marcadas e a barra para engadilas á lista Morning de P1">
<img src="../../images/android/5-library.png" width="180" alt="A lapela Biblioteca nun teléfono: as pistas da categoría Music, cada unha cunha caixa para marcar e uns auriculares para preescoitar, e debaixo a tarxeta de preescoita coa forma de onda da pista e os seus marcadores">
</p>

- *Directo:* cada reprodutor como unha tarxeta, e a cartucheira. Cada tarxeta
  ten, de arriba abaixo, o nome do reprodutor, o estado, Single/Cont e CUE; a
  portada, o título e o artista e a pista seguinte; a forma de onda (un toque
  busca); despois o tempo transcorrido e a duración da pista co tempo que
  queda en díxitos grandes sobre o transporte (Play/Seguinte, Anterior,
  Pausa, Stop, Reiniciar, Stop con fundido, Stop ao final da actual) con
  botóns do tamaño dun dedo, e un vúmetro fino co fader de volume (o nivel do
  fader móstrase debaixo del). O tempo que queda remata onde rematan os
  botóns, máis pequeno para unha pista longa. Na maioría dos teléfonos o
  vúmetro mostra a súa escala á esquerda, e nun máis ancho (arredor de 406 dp
  ou máis) o fader mostra tamén os seus niveis; nun teléfono estreito (menos
  de arredor de 384 dp) ningún dos dous o fai, e os botóns fanse un pouco máis
  pequenos para que caiban. Os botóns funcionan mentres outro dedo suxeita o
  fader, e un toque conta aínda que o dedo ruede un pouco ou quede un momento
  apoiado. En vertical as tarxetas están arriba, unha enteira e o comezo da
  seguinte (desprázate para ver máis), e a cartucheira debaixo. De lado, cada
  reprodutor é unha fila á esquerda (o seu nome e estado, o título, o tempo
  que queda, Play/Seguinte, Pausa e Stop; toca a fila noutro punto para abrir
  o reprodutor en Listas) e a cartucheira está á dereita. Toca
  **CARTUCHEIRA** na súa cabeceira para pregala: só queda a cabeceira, na
  parte de abaixo (as lapelas de páxina e *Paralo todo* seguen funcionando), e
  os reprodutores collen o sitio; tócaa de novo para despregala. A lapela
  Cartucheira mostra sempre a cartucheira enteira. Unha páxina de cartuchos
  demasiado ancha para o espazo móstrase en menos columnas, na mesma orde.
- *Cartucheira:* a páxina de cartuchos que se ve, ocupando a pantalla.
  Desliza cara aos lados para mostrar a páxina seguinte ou a anterior. En
  vertical, unha páxina máis ancha ca alta xírase (8 × 2 móstrase como 2 × 8),
  na mesma orde.
- *Listas:* os chips de arriba (P1, P2…) escollen o reprodutor; debaixo
  están as súas lapelas de listas (+ crea unha nova), a lista de pistas e, na
  parte de abaixo (á dereita cando o teléfono está de lado), a tarxeta do
  reprodutor cunha forma de onda máis grande para preparar unha pista (faise
  máis curta cando falta sitio, así que os botóns sempre se ven). Unha
  pulsación longa nunha fila abre o seu menú, que tamén ten **Subir**,
  **Baixar** e **Mover a outro reprodutor** (a pista vai xusto despois da
  pista seguinte dese reprodutor, na lista que mostra; a pista en antena
  queda). Arrastra unha fila polo seu asa ≡ para movela. O pé ten **Engadir**
  e, detrás de **⋯**, **Reiniciar**. Unha pulsación longa na forma de onda
  pon un marcador nese punto (cue-in, fin da intro, inicio do outro, MIX,
  cue-out) ou quita un que puxeches; os marcadores edítanse só aquí, non en
  Directo.
- *Biblioteca:* a biblioteca de pistas e a música do dispositivo (consulta
  [Biblioteca](#library)).

Atrás desde outra pantalla vai a Directo; en Biblioteca primeiro volve á lista
de categorías (en Música do dispositivo, primeiro sobe un cartafol).

Cando nada se move na pantalla, a aplicación debúxaa unha vez por segundo
(polo reloxo) para aforrar batería; un vúmetro que aínda cae despois dunha
pausa ou dun stop segue debuxando ata que o vúmetro chega abaixo.

**O cadeado** protexe contra toques perdidos durante un programa. Bloqueado
(o cadeado pechado e aceso), os faders de volume só mostran o nivel e as
formas de onda nin buscan nin editan marcadores; aparecen atenuados. Un
pellizco e un arrastre aínda amplían e moven unha forma de onda, e Play, o
transporte, os cartuchos e as listas funcionan sempre. O bloqueo segue despois
dun reinicio e gárdase coa sesión, así que cargar outra sesión toma o bloqueo
desa sesión; a API remota e o MIDI non están bloqueados.

Un arrastre que comeza nun fader de volume móveo; un que comeza noutro sitio
desprázase, así que desprazarse nunca cambia un nivel en antena. Un toque
nunha forma de onda busca alí, un pellizco con dous dedos amplía arredor dos
teus dedos, un arrastre cun dedo move unha vista ampliada e **Vista
completa** mostra de novo a pista enteira. Nunha lista un dedo desprázase;
para mover unha pista, usa o seu asa ≡ ou o menú da fila.

Nunha **tablet** a disposición é a do escritorio (consulta
[Disposición](layout.md)), con obxectivos máis fáciles de tocar.

Unha pulsación longa abre o que abre un clic dereito no escritorio: o menú da
fila, os menús das cabeceiras, o menú dun cartucho (**Editar…** escolle o
ficheiro dun cartucho baleiro). Nun teléfono, Configuración enche a pantalla e
lista as súas seccións; toca unha para abrila, co nome de cada axuste sobre o
seu control. A frecha ou Atrás volve á lista, e Atrás de novo (ou **Pechar**)
pecha Configuración. Editar un cartucho abre directamente a súa sección.

Acerca de → **Licenzas de terceiros** abre os avisos das bibliotecas coas que
se constrúe a aplicación nun navegador ou visor de HTML.

## Biblioteca {#library}

Nun **teléfono** a lapela **Biblioteca** abre unha lista de categorías:
**Todas as pistas**, as túas categorías (cada subcategoría baixo a súa
categoría pai, coa súa cor e cantas pistas contén), **Sen categoría** e
**Música do dispositivo**. A biblioteca é a mesma que mostra a [xanela da
biblioteca](library.md) do escritorio: está nela cada ficheiro engadido a
unha lista, a un cartucho ou á biblioteca. As categorías créanse, renómeanse,
cámbiaselles a cor e elimínanse en **Configuración → Biblioteca**.

**As pistas dunha categoría.** Toca unha categoría para abrir as súas pistas
(coas das súas subcategorías). A frecha da súa cabeceira, ou Atrás, volve ás
categorías. Outra lapela e volver atopa a lista como a deixaches. Abrir unha
categoría desde as categorías comeza cunha busca baleira. A busca filtra a
lista por título, artista, álbum e as outras etiquetas que busca a xanela da
biblioteca; **Toda a biblioteca** busca en todas as categorías. Cada fila
mostra o título (ou o nome do ficheiro), o artista, a duración e as marcas da
xanela para un ficheiro que falta, ilexible ou aínda non analizado.

Toca filas para marcalas. Mentres hai algo marcado, a barra de Música do
dispositivo aparece na parte de abaixo: cantas, o reprodutor e a lista aos que
van (tócaa para escoller outros), **Engadir ao final** e **Engadir tras a
seguinte**, como en Música do dispositivo. As pistas van na orde na que as
marcaches, e engadir limpa as marcas. Volver ás categorías tamén as limpa.

**O menú da fila.** Unha pulsación longa nunha fila abre o seu menú: **Engadir
a ▸** (a lista dun reprodutor, ao final), **Preescoitar**, **Mover á
categoría ▸**, **Volver analizar** e **Quitar da biblioteca…**, que pregunta
como no escritorio (consulta [Quitar pistas](library.md#removing-tracks)).
Mentres hai filas marcadas, unha pulsación longa noutra fila márcaa tamén e o
menú actúa sobre cada fila marcada, na orde na que as marcaches;
**Preescoitar** non se ofrece entón. As marcas quedan despois da acción do
menú.

**Preescoita.** Os 🎧 á dereita dunha fila reproducen a súa pista desde o seu
cue-in na saída de preescoita da biblioteca; os 🎧 doutra fila cambian a esa
pista. Unha franxa fina sobre a lista mostra o título, o tempo que queda,
**Pausa**/**Play** e **Stop** mentres soa (ou está en pausa), o título
atenuado despois de **Stop**, e unha pista antes da primeira preescoita:
sempre conserva o seu sitio, así que as filas non se moven cando unha
preescoita comeza ou remata. Os 🎧 aparecen atenuados cando o ficheiro falta ou
non se pode reproducir.

**A tarxeta de preescoita.** Toca a franxa (fóra dos seus botóns) para
abrir a tarxeta baixo a lista, coa forma de onda da pista como na tarxeta
dun reprodutor en Listas; a frecha á dereita da franxa indica se está
aberta. Toca a franxa outra vez, ou preme Atrás, para pechala; a preescoita
continúa. A frecha da cabeceira da lista volve ás categorías de inmediato e
deixa a tarxeta aberta para a seguinte lista. Na tarxeta:

- un toque na forma de onda salta a ese punto ou, coa preescoita detida,
  fixa onde comeza **Preescoitar**;
- unha pulsación longa pon un marcador nese punto (cue-in, fin da intro,
  inicio do outro, MIX, cue-out) ou quita un que puxeches. Os marcadores
  son da pista, así que o cambio vese en todas as listas e no escritorio;
- un belisco amplía, un arrastre move unha vista ampliada e **Vista
  completa** volve mostrar toda a pista;
- baixo a forma de onda están os tempos, **Preescoitar**, **Pausa**,
  **Deter** e o medidor.

A lista queda enriba da tarxeta (á esquerda co teléfono deitado), así que os
🎧 doutra fila seguen cambiando a tarxeta a esa pista. O cadeado detén os
saltos e os cambios de marcadores da forma de onda, como nos reprodutores.
A vista previa para cando a súa pista sae da
biblioteca e cando saes da lapela Biblioteca (por outra lapela, ou con Atrás);
nas categorías ou en Música do dispositivo continúa, pero a súa franxa só se
mostra na lista de pistas.

A diferenza do escritorio, Android (un teléfono ou unha tablet) reproduce a
preescoita da biblioteca na saída predeterminada cando non se escolle ningunha
saída para ela: en **Configuración → Saídas de audio → Biblioteca** a opción
baleira le *Predeterminado do sistema*, e o seu **Probar Cue** soa alí.
Escolle outra saída alí (uns auriculares con cable, por exemplo) para
preescoitar lonxe do aire. A saída predeterminada adoita ser tamén a dos
reprodutores e a da cartucheira, así que a preescoita pode oírse en antena:
mentres algo soa nesa saída, os 🎧 preguntan primeiro (**Preescoitar
igualmente** ou **Cancelar**), e unha preescoita que non confirmaches para
cando algo comeza alí, como fai o CUE dun reprodutor (consulta [CUE
(preescoita)](players.md#cue-pre-listen)).

**Engadir música do dispositivo a unha categoría.** O **＋** da cabeceira dunha
lista de pistas abre Música do dispositivo para esa categoría. A súa barra
só ten entón **Engadir a &lt;categoría&gt;** (**Engadir a Sen categoría** desde
*Todas as pistas* ou *Sen categoría*), que engade os ficheiros marcados á
biblioteca nesa categoría e volve á lista; un ficheiro que xa está na
biblioteca únese a ela se non ten categoría e conserva a que ten noutro caso,
como no escritorio. Música do dispositivo conserva as súas marcas entre
visitas, así que os ficheiros marcados nunha visita anterior engádense tamén:
comproba antes a conta da barra. **Volver á lista**, arriba, ou Atrás volve á
lista sen engadir. Música do dispositivo aberta desde as categorías ten
**Volver ás categorías** arriba en cambio.

**O cartafol do dispositivo dunha categoría.** Unha categoría pode tomar os
seus ficheiros dun cartafol da música do dispositivo. No seu menú (unha
pulsación longa nela en **Configuración → Biblioteca**, ou na xanela da
biblioteca nunha tablet), **Usar un cartafol do dispositivo…** abre Música do
dispositivo para escoller o cartafol (**Usar este cartafol**); o menú da
categoría nómeo entón, sobre os seus elementos de cartafol. Nada vixía o
cartafol: cando copiaches música nova alí, toca o **↻** da cabeceira da lista
de pistas da categoría (ou **Reler o cartafol do dispositivo** no seu menú).
Cada ficheiro que Música do dispositivo lista no cartafol e nos seus
subcartafoles engádese á categoría, coas regras de arriba (un ficheiro que xa
está noutra categoría conserva a que ten), e un aviso di cantos ficheiros se
leron. Os ficheiros que saíron do cartafol quedan na biblioteca, marcados como ausentes, e volven á normalidade cando
regresan a el (cando Música do dispositivo non lista nada, por exemplo sen
permiso, non se marca nada). **Cambiar o
cartafol do dispositivo…** e **Esquecer o cartafol do dispositivo** están no
mesmo menú; un cartafol pertence a unha soa categoría.

### Música do dispositivo {#device-music}

**Música do dispositivo** mostra os ficheiros de audio do dispositivo por
**Cartafoles** (coa ruta sobre a lista; toca unha parte para subir), **Artistas**
ou **Álbums**, e unha busca por título, artista, álbum e nome de ficheiro. Cada
fila mostra o título, o artista e a duración; toca filas para marcalas.
Mentres hai algo marcado, unha barra na parte de abaixo mostra cantas, o
reprodutor e a lista aos que van (tócaa para escoller outros), **Engadir ao
final** e, cando a pista seguinte dese reprodutor está nesa lista, **Engadir
tras a seguinte**. As pistas van na orde na que as marcaches. Un ficheiro do
dispositivo non ten preescoita.

**Engadir** no pé dunha lista, o ficheiro dun cartucho e Configuración →
Listas → Cartafol de música abren Música do dispositivo nun teléfono (nunha
tablet, a xanela do **navegador de música**, por cartafol). Marca ficheiros e
toca **Engadir**, toca un ficheiro para un cartucho, ou **Usar este
cartafol**; **Cancelar** ou Atrás volve onde estabas. Copia música ao
dispositivo (por exemplo ao seu cartafol *Music*) para vela alí. Fauste Player
só le a túa música: as etiquetas e as portadas non se escriben, e as listas e
as páxinas de cartuchos aínda non se importan nin se exportan en Android.

Nunha **tablet** a biblioteca é a [xanela da biblioteca](library.md) do
escritorio, aberta desde a barra superior.

## Saídas {#outputs}

Configuración → Saídas de audio lista as saídas do teléfono por tipo: o
altofalante, uns auriculares con cable, unha interface USB, un altofalante
Bluetooth («JBL Flip (Bluetooth)»). Cada reprodutor, a cartucheira e os seus
CUE escollen unha, como no escritorio, e tamén a preescoita da biblioteca
(consulta [Biblioteca](#library)). Un altofalante Bluetooth que se apaga e se
acende de novo atópase outra vez pola súa ruta; mentres non está, os seus
reprodutores compórtanse como con un dispositivo ausente no escritorio. Que
dúas saídas poidan soar á vez depende do teléfono.

## Non está en Android {#not-on-android}

As superficies de control MIDI, o control remoto, a edición de etiquetas e
portadas, os ficheiros de listas e de páxinas de cartuchos, e a automatización
(o musicalizador e Configuración → Automatización) son funcións do
escritorio por agora. Un teclado enchufado a unha tablet funciona como no
escritorio (consulta [Teclado](keyboard.md)).
