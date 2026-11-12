# Distribución

Organiza la ventana principal para tu turno: da a los players y a la
cartuchera el tamaño que quieras, pon la cartuchera debajo de los players o
a su lado, oculta los paneles que no uses y maximiza uno un momento. La
distribución se guarda con la sesión y vuelve en el siguiente arranque; una
instalación nueva empieza con todos los players del mismo ancho y la
cartuchera debajo de ellos.

## Mostrar y ocultar paneles {#show-and-hide-panels}

Los mosaicos de la izquierda de la barra superior, **Parrilla** y
**Música** (la [parrilla](automation.md#the-schedule-column) y el
[musicalizador](automation.md#the-music-filler), ocultos los dos hasta que
los muestras), **P1** … **Pn** y **Cartuchera**, indican qué paneles hay en pantalla: un mosaico encendido es
un panel visible. Haz clic en un mosaico para ocultar o mostrar su panel.
Con muchos players, los mosaicos se desplazan hacia los lados (mueve la
rueda del ratón sobre ellos) y el botón de distribución sigue a la vista.
Los demás paneles ocupan el espacio que queda libre, y un player que vuelve
a mostrarse recupera su ancho.

Ocultar solo quita un panel de la pantalla. Un player oculto sigue
reproduciendo, y su tecla numérica, el MIDI, la API remota, su ventana CUE y
la automatización lo siguen controlando. Un mosaico muestra el estado de su
panel, esté visible u oculto: un punto rojo mientras un player reproduce (o
la cartuchera mientras suena un botón) y dos barras ámbar, como un botón de
pausa, mientras un player está en pausa.

Al ocultar un player se borra su pista seleccionada, así que `Delete` no
puede quitar una pista que no ves. Los archivos que sueltes en la ventana
lejos de una lista van al último player que usaste o, si no, al primer
player visible; con todos los players ocultos se rechazan, y la barra de
estado lo dice.

El mosaico **Biblioteca**, a continuación de los demás, abre y cierra la
[ventana de la biblioteca](library.md), que flota sobre los paneles; no es
un panel, así que la regla del último mosaico encendido, más abajo, no se le
aplica. No se muestra en un teléfono.

Siempre queda al menos un panel visible: el último mosaico encendido no se
puede pulsar. Mientras la cartuchera está oculta, el atajo que la pliega o
la despliega no hace nada; haz clic en **Cartuchera** para mostrarla.

## Cambiar el tamaño de los players {#resize-the-players}

Arrastra el hueco entre dos players para pasar ancho de uno a otro. Los
demás players no se mueven, y ningún player queda más estrecho de lo que
necesitan sus controles. Haz doble clic en el hueco para que todos los
players vuelvan a tener el mismo ancho.

Cuando la ventana es demasiado estrecha para todos los players visibles en
su tamaño mínimo, la fila de players se desplaza hacia los lados, como
siempre.

## Mover y redimensionar la cartuchera {#move-and-resize-the-cartwall}

La cartuchera está debajo de los players, o a la izquierda o a la derecha de
la ventana, con toda su altura: elige **Cartuchera abajo**, **Cartuchera a
la izquierda** o **Cartuchera a la derecha** en [el menú de distribución](#the-layout-menu).
Debajo de los players tiene la altura que necesitan sus filas, como mucho
seis décimas de la altura entre la barra superior y la barra de estado; a
su lado ocupa al principio tres décimas del ancho.

La cartuchera nunca es más estrecha que su cabecera (**Parar todo**, su
título y una pestaña de página) ni que sus columnas de botones en su tamaño
mínimo. Cuando la ventana es demasiado estrecha para ella y para todos los
players visibles en su tamaño mínimo, la fila de players se desplaza hacia
los lados.

Arrastra el hueco entre la cartuchera y los players para hacerla mayor o
menor: arriba o abajo cuando está debajo de los players, hacia los lados
cuando está a su lado. Los players conservan el espacio que necesitan sus
controles. Haz doble clic en el hueco para volver al tamaño inicial: tan
alta como necesiten sus filas debajo de los players, tres décimas del ancho
a su lado. El tamaño se guarda con la distribución; al mover la cartuchera,
vuelve a empezar con su tamaño inicial.

Cuando cambias el tamaño de la cartuchera, o mientras está al lado de los
players, sus botones crecen para llenarla; los nombres y los tiempos
conservan su tamaño, en el centro de cada botón. Un nombre se parte entre
sus palabras; una palabra demasiado larga para su botón acaba en puntos
suspensivos (…) en lugar de partirse en varias líneas. Cuando la cartuchera
es menor de lo que necesitan sus filas y columnas con los botones más
pequeños, se desplaza.

Haz clic en **CARTUCHERA** para plegarla. Debajo de los players solo queda
su cabecera; a su lado se convierte en una franja estrecha de la altura de
la ventana, con una flecha para desplegarla de nuevo y, mientras suena un
botón, un punto rojo. Desplegada, recupera su tamaño. Mientras está
plegada, el hueco a su lado no hace nada.

## Maximizar un panel {#maximize-a-panel}

La cabecera de cada player, y la de la cartuchera, acaba con dos botones
pequeños: **Ocultar** (un signo menos) y **Maximizar** (esquinas que apuntan
hacia fuera; **Restaurar**, con las esquinas hacia dentro, en el panel
maximizado). Pasa el cursor por encima para ver el nombre. Se atenúan
mientras la distribución está bloqueada; **Ocultar**, en el último panel
visible; y **Maximizar**, cuando hay un solo panel visible y con la
cartuchera plegada. Las mismas dos acciones están en el menú del clic
derecho sobre el fondo de la cabecera (no sobre uno de sus botones):

- **Maximizar** da a ese panel todo el espacio: los demás paneles se
  reducen a su tamaño mínimo. Es solo por un momento y no se guarda.
  **Restaurar**, en el mismo menú, lo termina, y también lo terminan ocultar
  o mostrar un panel, bloquear la distribución, mover la cartuchera, cambiar
  el número de players y arrastrar o hacer doble clic en un hueco entre
  paneles; vuelven los tamaños de antes. Una cartuchera plegada no se puede
  maximizar, y plegar una cartuchera maximizada termina la maximización.
- **Ocultar** oculta el panel, igual que su mosaico en la barra superior.

## El menú de distribución {#the-layout-menu}

El botón de distribución, a la derecha de los mosaicos, abre un menú con:

- **Cartuchera abajo**, **Cartuchera a la izquierda**, **Cartuchera a la
  derecha**: dónde está la cartuchera;
- **Bloquear la distribución**: mientras está marcada, nada cambia la
  distribución por accidente: los mosaicos, las demás entradas de este menú,
  los huecos entre los paneles y los botones **Maximizar** y **Ocultar** de
  las cabeceras no hacen nada. La reproducción, y plegar o desplegar la
  cartuchera, funcionan como siempre. El bloqueo se guarda con la
  distribución;
- **Igualar el ancho de los players**: todos los players con el mismo ancho;
- **Restablecer la distribución**: vuelve a la distribución de una
  instalación nueva.
