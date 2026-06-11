export const es = {

  appName: "ClassMate",

  signOut: "Cerrar sesión",

  loading: "Cargando ClassMate...",

  nav: {

    dashboard: "Panel",

    myCourses: "Mis cursos",

    courses: "Cursos",

    gradebook: "Calificaciones",

    quizzes: "Cuestionarios",

    submissions: "Entregas",

    schedule: "Horario",

    announcements: "Anuncios",

    forums: "Foros",

    hub: "Aula en vivo",

    certificates: "Certificados",

    parent: "Padres",

    users: "Usuarios",

    settings: "Ajustes",

    sessions: "Sesiones",

    groups: "Grupos",

    fees: "Libro de caja",

    help: "Ayuda",

  },

  tenancy: {

    activeSchool: "Escuela activa",

  },

  login: {

    title: "Inicia sesión en tu espacio local",

    email: "Correo",

    password: "Contraseña",

    submit: "Entrar",

  },

  student: {

    title: "Mis cursos",

    subtitle: "Tus clases, calificaciones y materiales.",

    average: "Promedio",

    assignments: "Tareas",

    graded: "Calificadas",

    upcoming: "Próximo trabajo",

    materials: "Materiales",

    rubricBreakdown: "Desglose de rúbrica",

    noCourses: "Aún no estás inscrito en ningún curso.",

    teacher: "Profesor",

  },

  appearance: {

    title: "Apariencia e idioma",

    schoolName: "Nombre de la escuela",

    theme: "Tema",

    themeDefault: "Predeterminado",

    themeHighContrast: "Alto contraste",

    fontScale: "Tamaño de texto",

    accent: "Color de acento",

    locale: "Idioma",

    save: "Guardar apariencia",

    saved: "Apariencia guardada.",

  },

  settings: {

    title: "Ajustes y sincronización",

    subtitle: "Video, copias de seguridad, sincronización e integraciones.",

    saved: "Ajustes guardados.",

    video: {

      title: "Video en vivo",

      galeneTitle: "Galene (ligero)",

      installed: "Instalado",

      running: "En ejecución",

      yes: "Sí",

      no: "No",

      installHint: "Instalar Galene:",

      refresh: "Actualizar",

      start: "Iniciar video",

      stop: "Detener",

      externalUrl: "URL de video externa alternativa",

      plugnmeetTitle: "plugNmeet (aula completa)",

      plugnmeetHint: "Autoaloja plugNmeet y configura la URL del servidor. Se usa si Galene no está disponible.",

      plugnmeetBase: "URL del servidor plugNmeet",

      plugnmeetRoom: "ID de sala predeterminado",

      saveVideo: "Guardar ajustes de video",

    },

    sync: {

      title: "Sincronización LAN",

      hint: "Sincroniza bases de datos entre equipos en la misma red Wi‑Fi. Usa el mismo token en ambos.",

      server: "Servidor",

      running: "Activo",

      stopped: "Detenido",

      port: "Puerto",

      token: "Token de sincronización",

      saveToken: "Guardar token",

      start: "Iniciar servidor",

      stopServer: "Detener servidor",

      peerUrl: "URL del equipo",

      pull: "Traer del equipo",

      push: "Enviar al equipo",

      started: "Servidor LAN iniciado.",

      stoppedMsg: "Servidor LAN detenido.",

      tokenSaved: "Token actualizado.",

      publicTitle: "Dominio público (HTTPS)",

      publicHint: "Apunta tu dominio a este PC con un proxy inverso (ver DEPLOY.md). La ruta del Hub es /hub por defecto.",

      publicBaseUrl: "URL base pública",

      publicHubPath: "Ruta del Hub",

      publicSave: "Guardar URLs públicas",

      publicSaved: "URLs públicas guardadas.",

    },

    backup: {

      title: "Copia y restauración",

      hint: "Exporta una copia JSON completa para migración o sincronización externa.",

      export: "Exportar copia",

      import: "Importar copia",

      restored: "Copia restaurada.",

      autoTitle: "Copia automática programada",

      autoHint: "Guarda copias en la carpeta de datos de la aplicación según un horario.",

      enable: "Activar copia automática",

      interval: "Intervalo",

      daily: "Diario",

      weekly: "Semanal",

      off: "Desactivado",

      maxKeep: "Conservar copias más recientes",

      lastRun: "Última copia",

      nextRun: "Próxima",

      never: "Nunca",

      backupNow: "Copiar ahora",

      saveAuto: "Guardar horario",

      autoSaved: "Ajustes de copia guardados.",

      backupDone: "Copia guardada.",

      folder: "Carpeta de copias",

      restore: "Restaurar",

      confirmRestore: "¿Restaurar esta copia? Se reemplazarán los datos actuales.",

      cloudTitle: "Envío de copia a la nube",

      cloudHint: "Envía la copia JSON a un endpoint de sincronización remoto.",

      cloudUrl: "URL del endpoint",

      cloudToken: "Token de sincronización",

      pushCloud: "Enviar copia a la nube",

    },

    whatsapp: {

      title: "Compartir por WhatsApp",

      hint: "Código de país predeterminado para números locales (solo dígitos).",

      countryCode: "Código de país",

    },

    whatsappBusiness: {

      title: "WhatsApp Business API",

      hint: "Conecta la API de Meta para mensajes automáticos. Inicia el servidor LAN sync para el webhook.",

      apiVersion: "Versión Graph API",

      phoneNumberId: "ID del número",

      accessToken: "Token de acceso",

      webhookToken: "Token de verificación webhook",

      webhookUrl: "URL del webhook",

      save: "Guardar API",

      test: "Probar conexión",

      configured: "API configurada",

      notConfigured: "API no configurada",

      templateTitle: "Plantilla de recordatorio",

      templateHint: "Use una plantilla aprobada por Meta con 4 variables: nombre del alumno, curso, tarea y fecha límite.",

      templateName: "Nombre de plantilla",

      templateLanguage: "Código de idioma",

      templateSave: "Guardar plantilla",

      templateSaved: "Plantilla guardada.",

      groupInviteTemplateTitle: "Plantilla de invitación a grupo",

      groupInviteTemplateHint: "Plantilla aprobada por Meta con parámetro group_id en el cuerpo para invitaciones nativas.",

      groupInviteTemplateSave: "Guardar plantilla de invitación",

      complianceTitle: "Cumplimiento regional",

      complianceHint: "Baja automática cuando el usuario responde STOP. Exporte datos WhatsApp por usuario en Usuarios.",

      autoUnsubscribe: "Baja automática por palabras clave",

      unsubscribeKeywords: "Palabras de baja (separadas por coma)",

      complianceSave: "Guardar cumplimiento",

      complianceSaved: "Cumplimiento guardado.",

    },

    schools: {

      title: "Multi-escuela",

      hint: "Administre campus y asigne usuarios. Los datos dependen de la escuela activa en la barra lateral.",

      activeTitle: "Escuela activa",

      createTitle: "Crear escuela",

      membersTitle: "Miembros",

      name: "Nombre",

      code: "Código",

      saveActive: "Guardar escuela activa",

      create: "Crear escuela",

      created: "Escuela creada.",

      updated: "Escuela actualizada.",

      memberEmail: "Email del usuario",

      addMember: "Agregar miembro",

      memberAdded: "Miembro agregado.",

      memberRemoved: "Miembro eliminado.",

      memberName: "Usuario",

      memberRole: "Rol",

      removeMember: "Quitar",

    },

    push: {

      title: "Push móvil (FCM / APNs)",

      hint: "Envíe recordatorios de tareas a dispositivos registrados. Los estudiantes del Class Hub registran vía POST /api/student/push/register.",

      enable: "Activar notificaciones push",

      fcmTitle: "Firebase Cloud Messaging",

      fcmProject: "ID del proyecto FCM",

      fcmServiceAccount: "JSON de cuenta de servicio",

      apnsTitle: "Apple Push Notification service",

      apnsKeyId: "ID de clave APNs",

      apnsTeamId: "ID del equipo Apple",

      apnsBundleId: "Bundle ID",

      apnsPrivateKey: "Clave privada .p8 APNs",

      apnsSandbox: "Usar sandbox APNs",

      save: "Guardar push",

      saved: "Push guardado.",

      testTitle: "Probar push",

      testPlatform: "Plataforma",

      testToken: "Token del dispositivo",

      testSend: "Enviar prueba",

      remindersTitle: "Recordatorios de tareas",

      configured: "Push configurado",

      notConfigured: "Push no configurado",

      deviceCount: "{count} dispositivos registrados",

      lastRun: "Última ejecución",

      remindersEnable: "Recordatorios diarios de entregas",

      remindersSave: "Guardar recordatorios",

      remindersSaved: "Recordatorios guardados.",

      remindersNow: "Ejecutar ahora",

      remindersRun: "Enviados {sent}, fallidos {failed}",

      logTitle: "Registro reciente",

    },

    smtp: {

      title: "Correo SMTP",

      hint: "Envía resúmenes semanales a padres por correo.",

      host: "Servidor SMTP",

      port: "Puerto",

      username: "Usuario",

      from: "Remitente",

      password: "Contraseña",

      useTls: "Usar STARTTLS",

      save: "Guardar SMTP",

      test: "Enviar prueba",

      testTo: "Destinatario de prueba",

      configured: "SMTP configurado",

      notConfigured: "SMTP no configurado",

      logTitle: "Registro de correo",

      digestTitle: "Resumen programado para padres",

      digestHint: "Envía automáticamente el resumen de calificaciones a padres vinculados al iniciar la app.",

      digestEnable: "Activar envío programado",

      digestInterval: "Frecuencia",

      digestDaily: "Diario",

      digestWeekly: "Semanal",

      digestOff: "Desactivado",

      digestSave: "Guardar programación",

      digestNow: "Enviar todos ahora",

      digestSaved: "Programación guardada.",

      digestDone: "Enviados {sent}, fallidos {failed}, omitidos {skipped}.",

      digestLastRun: "Último envío",

      digestNextRun: "Próximo envío",

      digestNever: "Nunca",

      parentCount: "{count} padres vinculados",

    },

    oneroster: {

      title: "Importar CSV OneRoster",

      import: "Importar usuarios",

      result: "Importados {imported}, omitidos {skipped}",

    },

    lti: {

      title: "Herramientas LTI 1.3",

      add: "Agregar herramienta",

    },

  },

  fees: {

    title: "Libro de caja",

    subtitle: "Registro de ingresos y gastos solo para administradores: cuotas, salarios y costos.",

    totalIncome: "Ingresos totales",

    totalExpense: "Gastos totales",

    balance: "Saldo",

    entries: "Registros",

    filterTitle: "Filtro por fecha",

    fromDate: "Desde",

    toDate: "Hasta",

    applyFilter: "Aplicar",

    exportCsv: "Exportar CSV",

    addEntry: "Registrar transacción",

    direction: "Tipo",

    income: "Ingreso",

    expense: "Gasto",

    category: "Categoría",

    categoryStudentFee: "Cuota estudiante",

    categoryOtherIncome: "Otro ingreso",

    categoryTeacherSalary: "Salario docente",

    categoryOtherExpense: "Otro gasto",

    amount: "Monto",

    paymentMethod: "Forma de pago",

    methodCash: "Efectivo",

    methodBank: "Transferencia",

    methodCheque: "Cheque",

    methodOnline: "En línea",

    linkedUser: "Usuario vinculado",

    linkedCourse: "Curso vinculado",

    none: "Ninguno",

    entryDate: "Fecha",

    reference: "Referencia",

    description: "Descripción",

    saveEntry: "Guardar",

    entrySaved: "Registro guardado.",

    invalidAmount: "Ingrese un monto válido.",

    deleteConfirm: "¿Eliminar este registro?",

    delete: "Eliminar",

    ledgerTitle: "Libro mayor",

    noEntries: "Sin registros aún.",

    integrationTitle: "Integración y moneda",

    integrationHint: "Invoice Ninja autoalojado (AGPL, código abierto) opcional para facturación. ClassMate mantiene el libro local; exporte CSV o conecte Invoice Ninja.",

    currency: "Moneda predeterminada",

    invoiceNinjaUrl: "URL de Invoice Ninja",

    invoiceNinjaToken: "Token API de Invoice Ninja",

    saveSettings: "Guardar ajustes",

    settingsSaved: "Ajustes del libro guardados.",

    testIntegration: "Probar Invoice Ninja",

  },

  help: {

    title: "Ayuda y configuración",

    subtitle: "Instala ClassMate, conecta tu dominio, sincroniza datos y configura WhatsApp.",

    toc: { download: "Descargar", local: "Local", domain: "Dominio", sync: "Sync", hub: "Class Hub", whatsapp: "WhatsApp", security: "Seguridad" },

    download: {

      title: "Descargar ClassMate (Windows)",

      body: "Instala la app de escritorio en el PC del administrador. SQLite incluido.",

      web: "Abrir página de descarga",

      direct: "Descargar instalador (.exe)",

      build: "O compila desde el código (Node.js 20+ y Rust):",

      output: "Salida: src-tauri\\target\\release\\bundle\\nsis\\ClassMate_0.26.0_x64-setup.exe",

    },

    local: {

      title: "Uso local (sin dominio)",

      step1: "Inicia sesión y cambia las contraseñas demo en Usuarios.",

      step2: "Crea cursos, matrículas, tareas y materiales.",

      step3: "Inicia Class Hub para sesiones en vivo.",

      step4: "Exporta copias de seguridad en Ajustes.",

      step5: "Los datos se guardan en SQLite en este equipo.",

    },

    demo: { role: "Rol", email: "Correo", password: "Contraseña" },

    domain: {

      title: "Dominio público (ej. cm.codes-ai.uk)",

      body: "Usa un VPS y proxy inverso para HTTPS.",

      step1: "DNS GoDaddy: registro A cm → IP del servidor.",

      step2: "Ajustes → sync LAN → URL pública https://tu-dominio",

      step3: "En el servidor: nginx + certbot. Ver DEPLOY.md.",

      configured: "URL pública:",

      notConfigured: "URL pública no configurada — Ajustes → sync LAN.",

    },

    sync: {

      title: "Sincronización LAN / servidor",

      body: "Mantén la misma base de datos en PC y servidor.",

      item1: "Inicia el servidor sync (escritorio o classmate-server en Linux).",

      item2: "Usa el mismo token de sync en ambos equipos.",

      item3: "Push/Pull en Ajustes con https://tu-dominio/api/sync/",

      status: "Servidor sync",

      running: "activo",

      stopped: "detenido",

    },

    hub: {

      title: "Class Hub",

      body: "Portal en vivo para alumnos.",

      step1: "Class Hub → curso → Iniciar hub.",

      step2: "Comparte la URL de acceso.",

      step3: "Los alumnos usan el PIN de la sesión.",

    },

    whatsapp: {

      title: "WhatsApp Business API",

      body: "Mensajería automática opcional.",

      step1: "Ajustes → WhatsApp Business API — credenciales Meta.",

      step2: "Pega la URL del webhook en Meta.",

      step3: "Suscríbete a los campos de webhook de grupo y mensajes.",

      step4: "Página Groups — consentimiento y grupos.",

    },

    security: {

      title: "Antes de producción",

      item1: "Cambia todas las contraseñas demo.",

      item2: "Usa un token sync fuerte.",

      item3: "Solo expón Hub y API por HTTPS.",

    },

  },

} as const;

