# 💽 DiskMan (Rust UDisks2 + Rofi)

Una utilidad ligera y ultrarrápida desarrollada en **Rust** para la gestión de volúmenes extraíbles en entornos Linux con Wayland y Hyprland (o cualquier gestor de ventanas). Diseñada para prescindir de pesados gestores de archivos y ofrecer control directo mediante menús flotantes.

## 🚀 Características

* **Rendimiento nativo:** Escrito en Rust, sin tiempos de espera ni consumo innecesario de recursos.
* **Integración con Rofi:** Despliega un menú interactivo en pantalla para montar y desmontar unidades con un solo atajo de teclado.
* **Filtrado Inteligente:** Ignora automáticamente las unidades padre no montables y lista únicamente las particiones o volúmenes extraíbles válidos.
* **Sin Privilegios de Root:** Utiliza `udisksctl` y políticas de seguridad modernas (`polkit`) para operar de forma segura bajo el usuario actual.
* **Notificaciones de Escritorio:** Feedback visual instantáneo mediante `notify-send`.

---

## 🛠️ Requisitos del Sistema

* **Arch Linux** (o cualquier distro compatible con UDisks2).
* **Rust toolchain** (instalado vía `rustup`).
* **Rofi** (modo dmenu / Wayland fork compatible).
* **`udisksctl`** (suele venir preinstalado con el paquete `udisks2`).
* **`notify-send`** (paquete `libnotify`).
* Nautilus (explorador de archivos)
