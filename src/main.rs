use serde::Deserialize;
use std::io::Write;
use std::process::{Command, Stdio};

#[derive(Deserialize, Debug)]
struct LsblkOutput {
    blockdevices: Vec<Dispositivo>,
}

#[derive(Deserialize, Debug)]
struct Dispositivo {
    name: String,
    size: String,
    rm: bool,
    #[serde(default)]
    mountpoints: Option<Vec<Option<String>>>,
    #[serde(default)]
    children: Option<Vec<Dispositivo>>,
}

struct InfoDispositivo {
    name: String,
    size: String,
    montado: bool,
}

fn main() {
    let dispositivos = match listar_extraibles() {
        Ok(lista) => lista,
        Err(e) => {
            enviar_notificacion("Error DiskMan", &format!("Error al listar discos: {}", e), "critical");
            return;
        }
    };

    if dispositivos.is_empty() {
        enviar_notificacion("DiskMan", "No hay dispositivos extraíbles conectados.", "normal");
        return;
    }

    let mut opciones_texto = String::new();
    for d in &dispositivos {
        let accion = if d.montado { "Desmontar" } else { "Montar" };
        let estado = if d.montado { "Montado" } else { "Desmontado" };
        let linea = format!("{} ➔ /dev/{} ({}) [{}]\n", accion, d.name, d.size, estado);
        opciones_texto.push_str(&linea);
    }

    let seleccion = match lanzar_rofi(&opciones_texto) {
        Ok(res) => res,
        Err(_) => return, 
    };

    if seleccion.trim().is_empty() {
        return;
    }

    if let Some(dispositivo) = extraer_nombre_dispositivo(&seleccion) {
        let es_desmontar = seleccion.starts_with("Desmontar");
        let accion_str = if es_desmontar { "unmount" } else { "mount" };

        match gestionar_volumen(&dispositivo, accion_str) {
            Ok(_) => {
                let msg = format!("Dispositivo /dev/{} {} con éxito.", dispositivo, if es_desmontar { "desmontado" } else { "montado" });
                enviar_notificacion("DiskMan", &msg, "normal");
            }
            Err(e) => {
                let msg = format!("Fallo al gestionar /dev/{}: {}", dispositivo, e);
                enviar_notificacion("DiskMan", &msg, "critical");
            }
        }
    }
}


fn listar_extraibles() -> Result<Vec<InfoDispositivo>, Box<dyn std::error::Error>> {
    let output = Command::new("lsblk").arg("-J").output()?;
    let datos: LsblkOutput = serde_json::from_slice(&output.stdout)?;

    let mut lista = Vec::new();

    for disco in datos.blockdevices {
        if let Some(hijos) = disco.children {
            for hijo in hijos {
                if hijo.rm {
                    lista.push(mapear_dispositivo(&hijo));
                }
            }
        } else {
            if disco.rm {
                lista.push(mapear_dispositivo(&disco));
            }
        }
    }

    Ok(lista)
}
fn mapear_dispositivo(d: &Dispositivo) -> InfoDispositivo {
    let montado = match &d.mountpoints {
        Some(pts) => pts.iter().any(|p| p.is_some()),
        None => false,
    };

    InfoDispositivo {
        name: d.name.clone(),
        size: d.size.clone(),
        montado,
    }
}

fn lanzar_rofi(opciones: &str) -> Result<String, std::io::Error> {
    let mut child = Command::new("rofi")
        .args(["-dmenu", "-p", "💽 Almacenamiento"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(opciones.as_bytes())?;
    }

    let output = child.wait_with_output()?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn extraer_nombre_dispositivo(linea_rofi: &str) -> Option<String> {
    if let Some(idx) = linea_rofi.find("/dev/") {
        let resto = &linea_rofi[idx + 5..];
        let nombre: String = resto.chars().take_while(|c| c.is_alphanumeric()).collect();
        if !nombre.is_empty() {
            return Some(nombre);
        }
    }
    None
}

fn gestionar_volumen(nombre_particion: &str, accion: &str) -> Result<(), String> {
    let dispositivo_path = format!("/dev/{}", nombre_particion);

    let status = Command::new("udisksctl")
        .arg(accion)
        .arg("-b")
        .arg(&dispositivo_path)
        .status()
        .map_err(|e| format!("Error al invocar udisksctl: {}", e))?;

    if !status.success() {
        return Err(String::from("udisksctl devolvió un error (¿falta de permisos o contraseña de polkit?)"));
    }

    if accion == "mount" {
        if let Ok(user) = std::env::var("USER") {
            let ruta_montaje = format!("/run/media/{}", user);
            let _ = Command::new("nautilus")
                .arg("--new-window")
                .arg(ruta_montaje)
                .spawn();
        }
    }

    Ok(())
}

fn enviar_notificacion(titulo: &str, mensaje: &str, urgencia: &str) {
    let _ = Command::new("notify-send")
        .arg("-u")
        .arg(urgencia)
        .arg(titulo)
        .arg(mensaje)
        .status();
}
