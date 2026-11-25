.class public Lorg/garlic/client/MainActivity;
.super Landroid/app/Activity;
.source "MainActivity.java"
.implements Lorg/garlic/client/SecondaryButtons;


# instance fields
.field private account:Lorg/json/JSONObject;

.field private admin:Z

.field private final bg:I

.field private connection:Landroid/widget/TextView;

.field private content:Landroid/widget/LinearLayout;

.field private generation:I

.field private final handler:Landroid/os/Handler;

.field private header:Landroid/widget/LinearLayout;

.field private final ink:I

.field private key:Landroid/widget/EditText;

.field private loggedIn:Z

.field private message:Landroid/widget/TextView;

.field private final muted:I

.field private page:Landroid/widget/LinearLayout;

.field private power:Landroid/widget/Button;

.field private prefs:Landroid/content/SharedPreferences;

.field private roleVerified:Z

.field private routeInfo:Landroid/widget/TextView;

.field private server:Landroid/widget/EditText;

.field private starting:Z

.field private statsInfo:Landroid/widget/TextView;

.field private final statusPoll:Ljava/lang/Runnable;

.field private subscriptionInfo:Landroid/widget/TextView;

.field private final surface:I

.field private vpnPermissionGeneration:I

.field private final worker:Ljava/util/concurrent/ExecutorService;


# direct methods
.method static bridge synthetic -$$Nest$fgetconnection(Lorg/garlic/client/MainActivity;)Landroid/widget/TextView;
    .locals 0

    iget-object p0, p0, Lorg/garlic/client/MainActivity;->connection:Landroid/widget/TextView;

    return-object p0
.end method

.method static bridge synthetic -$$Nest$fgethandler(Lorg/garlic/client/MainActivity;)Landroid/os/Handler;
    .locals 0

    iget-object p0, p0, Lorg/garlic/client/MainActivity;->handler:Landroid/os/Handler;

    return-object p0
.end method

.method static bridge synthetic -$$Nest$fgetmessage(Lorg/garlic/client/MainActivity;)Landroid/widget/TextView;
    .locals 0

    iget-object p0, p0, Lorg/garlic/client/MainActivity;->message:Landroid/widget/TextView;

    return-object p0
.end method

.method static bridge synthetic -$$Nest$fgetpower(Lorg/garlic/client/MainActivity;)Landroid/widget/Button;
    .locals 0

    iget-object p0, p0, Lorg/garlic/client/MainActivity;->power:Landroid/widget/Button;

    return-object p0
.end method

.method static bridge synthetic -$$Nest$fgetprefs(Lorg/garlic/client/MainActivity;)Landroid/content/SharedPreferences;
    .locals 0

    iget-object p0, p0, Lorg/garlic/client/MainActivity;->prefs:Landroid/content/SharedPreferences;

    return-object p0
.end method

.method static bridge synthetic -$$Nest$fgetrouteInfo(Lorg/garlic/client/MainActivity;)Landroid/widget/TextView;
    .locals 0

    iget-object p0, p0, Lorg/garlic/client/MainActivity;->routeInfo:Landroid/widget/TextView;

    return-object p0
.end method

.method static bridge synthetic -$$Nest$fgetstarting(Lorg/garlic/client/MainActivity;)Z
    .locals 0

    iget-boolean p0, p0, Lorg/garlic/client/MainActivity;->starting:Z

    return p0
.end method

.method static bridge synthetic -$$Nest$fgetstatsInfo(Lorg/garlic/client/MainActivity;)Landroid/widget/TextView;
    .locals 0

    iget-object p0, p0, Lorg/garlic/client/MainActivity;->statsInfo:Landroid/widget/TextView;

    return-object p0
.end method

.method static bridge synthetic -$$Nest$fgetsubscriptionInfo(Lorg/garlic/client/MainActivity;)Landroid/widget/TextView;
    .locals 0

    iget-object p0, p0, Lorg/garlic/client/MainActivity;->subscriptionInfo:Landroid/widget/TextView;

    return-object p0
.end method

.method static bridge synthetic -$$Nest$fputstarting(Lorg/garlic/client/MainActivity;Z)V
    .locals 0

    iput-boolean p1, p0, Lorg/garlic/client/MainActivity;->starting:Z

    return-void
.end method

.method static bridge synthetic -$$Nest$msubscription(Lorg/garlic/client/MainActivity;)Ljava/lang/String;
    .locals 0

    invoke-direct {p0}, Lorg/garlic/client/MainActivity;->subscription()Ljava/lang/String;

    move-result-object p0

    return-object p0
.end method

.method static bridge synthetic -$$Nest$mupdateText(Lorg/garlic/client/MainActivity;Landroid/widget/TextView;Ljava/lang/String;)V
    .locals 0

    invoke-direct {p0, p1, p2}, Lorg/garlic/client/MainActivity;->updateText(Landroid/widget/TextView;Ljava/lang/String;)V

    return-void
.end method

.method public constructor <init>()V
    .locals 2

    .line 19
    invoke-direct {p0}, Landroid/app/Activity;-><init>()V

    .line 27
    const/16 v0, 0x17

    invoke-static {v0, v0, v0}, Landroid/graphics/Color;->rgb(III)I

    move-result v0

    iput v0, p0, Lorg/garlic/client/MainActivity;->bg:I

    const/16 v0, 0x23

    invoke-static {v0, v0, v0}, Landroid/graphics/Color;->rgb(III)I

    move-result v0

    iput v0, p0, Lorg/garlic/client/MainActivity;->surface:I

    const/16 v0, 0xaa

    invoke-static {v0, v0, v0}, Landroid/graphics/Color;->rgb(III)I

    move-result v0

    iput v0, p0, Lorg/garlic/client/MainActivity;->muted:I

    const/16 v0, 0xf5

    invoke-static {v0, v0, v0}, Landroid/graphics/Color;->rgb(III)I

    move-result v0

    iput v0, p0, Lorg/garlic/client/MainActivity;->ink:I

    .line 28
    invoke-static {}, Ljava/util/concurrent/Executors;->newSingleThreadExecutor()Ljava/util/concurrent/ExecutorService;

    move-result-object v0

    iput-object v0, p0, Lorg/garlic/client/MainActivity;->worker:Ljava/util/concurrent/ExecutorService;

    .line 29
    new-instance v0, Landroid/os/Handler;

    invoke-static {}, Landroid/os/Looper;->getMainLooper()Landroid/os/Looper;

    move-result-object v1

    invoke-direct {v0, v1}, Landroid/os/Handler;-><init>(Landroid/os/Looper;)V

    iput-object v0, p0, Lorg/garlic/client/MainActivity;->handler:Landroid/os/Handler;

    .line 36
    const/4 v0, 0x0

    iput-boolean v0, p0, Lorg/garlic/client/MainActivity;->admin:Z

    iput-boolean v0, p0, Lorg/garlic/client/MainActivity;->loggedIn:Z

    iput-boolean v0, p0, Lorg/garlic/client/MainActivity;->starting:Z

    iput-boolean v0, p0, Lorg/garlic/client/MainActivity;->roleVerified:Z

    .line 37
    iput v0, p0, Lorg/garlic/client/MainActivity;->generation:I

    .line 38
    const/4 v0, -0x1

    iput v0, p0, Lorg/garlic/client/MainActivity;->vpnPermissionGeneration:I

    .line 39
    new-instance v0, Lorg/garlic/client/MainActivity$1;

    invoke-direct {v0, p0}, Lorg/garlic/client/MainActivity$1;-><init>(Lorg/garlic/client/MainActivity;)V

    iput-object v0, p0, Lorg/garlic/client/MainActivity;->statusPoll:Ljava/lang/Runnable;

    return-void
.end method

.method private action(Ljava/lang/String;Ljava/lang/String;)Lorg/json/JSONObject;
    .locals 2

    .line 181
    new-instance v0, Lorg/json/JSONObject;

    invoke-direct {v0}, Lorg/json/JSONObject;-><init>()V

    :try_start_0
    const-string v1, "action"

    invoke-virtual {v0, v1, p1}, Lorg/json/JSONObject;->put(Ljava/lang/String;Ljava/lang/Object;)Lorg/json/JSONObject;

    const-string p1, "username"

    invoke-virtual {v0, p1, p2}, Lorg/json/JSONObject;->put(Ljava/lang/String;Ljava/lang/Object;)Lorg/json/JSONObject;
    :try_end_0
    .catch Ljava/lang/Exception; {:try_start_0 .. :try_end_0} :catch_0

    goto :goto_0

    :catch_0
    move-exception p1

    :goto_0
    return-object v0
.end method

.method public secondaryButton(Ljava/lang/String;)Landroid/widget/Button;
    .locals 1
    const/4 v0, 0x0
    invoke-direct {p0, p1, v0}, Lorg/garlic/client/MainActivity;->button(Ljava/lang/String;Z)Landroid/widget/Button;
    move-result-object v0
    return-object v0
.end method

.method private button(Ljava/lang/String;Z)Landroid/widget/Button;
    .locals 3

    .line 58
    new-instance v0, Landroid/widget/Button;

    invoke-direct {v0, p0}, Landroid/widget/Button;-><init>(Landroid/content/Context;)V

    invoke-virtual {v0, p1}, Landroid/widget/Button;->setText(Ljava/lang/CharSequence;)V

    const/4 p1, 0x0

    invoke-virtual {v0, p1}, Landroid/widget/Button;->setAllCaps(Z)V

    const/high16 v1, 0x41700000    # 15.0f

    invoke-virtual {v0, v1}, Landroid/widget/Button;->setTextSize(F)V

    if-eqz p2, :cond_0

    iget v1, p0, Lorg/garlic/client/MainActivity;->bg:I

    goto :goto_0

    :cond_0
    iget v1, p0, Lorg/garlic/client/MainActivity;->ink:I

    :goto_0
    invoke-virtual {v0, v1}, Landroid/widget/Button;->setTextColor(I)V

    if-eqz p2, :cond_1

    iget p2, p0, Lorg/garlic/client/MainActivity;->ink:I

    goto :goto_1

    :cond_1
    iget p2, p0, Lorg/garlic/client/MainActivity;->surface:I

    :goto_1
    const/16 v1, 0xe

    invoke-direct {p0, p2, v1}, Lorg/garlic/client/MainActivity;->shape(II)Landroid/graphics/drawable/GradientDrawable;

    move-result-object p2

    invoke-virtual {v0, p2}, Landroid/widget/Button;->setBackground(Landroid/graphics/drawable/Drawable;)V

    new-instance p2, Landroid/widget/LinearLayout$LayoutParams;

    const/16 v1, 0x34

    invoke-direct {p0, v1}, Lorg/garlic/client/MainActivity;->dp(I)I

    move-result v1

    const/4 v2, -0x1

    invoke-direct {p2, v2, v1}, Landroid/widget/LinearLayout$LayoutParams;-><init>(II)V

    const/4 v1, 0x6

    invoke-direct {p0, v1}, Lorg/garlic/client/MainActivity;->dp(I)I

    move-result v2

    invoke-direct {p0, v1}, Lorg/garlic/client/MainActivity;->dp(I)I

    move-result v1

    invoke-virtual {p2, p1, v2, p1, v1}, Landroid/widget/LinearLayout$LayoutParams;->setMargins(IIII)V

    invoke-virtual {v0, p2}, Landroid/widget/Button;->setLayoutParams(Landroid/view/ViewGroup$LayoutParams;)V

    return-object v0
.end method

.method private chooseApps()V
    .locals 7

    .line 119
    invoke-virtual {p0}, Lorg/garlic/client/MainActivity;->getPackageManager()Landroid/content/pm/PackageManager;

    move-result-object v0

    new-instance v1, Landroid/content/Intent;

    const-string v2, "android.intent.action.MAIN"

    invoke-direct {v1, v2}, Landroid/content/Intent;-><init>(Ljava/lang/String;)V

    const-string v2, "android.intent.category.LAUNCHER"

    invoke-virtual {v1, v2}, Landroid/content/Intent;->addCategory(Ljava/lang/String;)Landroid/content/Intent;

    move-result-object v1

    .line 120
    const/4 v2, 0x0

    invoke-virtual {v0, v1, v2}, Landroid/content/pm/PackageManager;->queryIntentActivities(Landroid/content/Intent;I)Ljava/util/List;

    move-result-object v1

    new-instance v3, Ljava/util/TreeMap;

    invoke-direct {v3}, Ljava/util/TreeMap;-><init>()V

    .line 121
    invoke-interface {v1}, Ljava/util/List;->iterator()Ljava/util/Iterator;

    move-result-object v1

    :cond_0
    :goto_0
    invoke-interface {v1}, Ljava/util/Iterator;->hasNext()Z

    move-result v4

    if-eqz v4, :cond_1

    invoke-interface {v1}, Ljava/util/Iterator;->next()Ljava/lang/Object;

    move-result-object v4

    check-cast v4, Landroid/content/pm/ResolveInfo;

    iget-object v5, v4, Landroid/content/pm/ResolveInfo;->activityInfo:Landroid/content/pm/ActivityInfo;

    iget-object v5, v5, Landroid/content/pm/ActivityInfo;->packageName:Ljava/lang/String;

    invoke-virtual {p0}, Lorg/garlic/client/MainActivity;->getPackageName()Ljava/lang/String;

    move-result-object v6

    invoke-virtual {v5, v6}, Ljava/lang/String;->equals(Ljava/lang/Object;)Z

    move-result v6

    if-nez v6, :cond_0

    invoke-virtual {v4, v0}, Landroid/content/pm/ResolveInfo;->loadLabel(Landroid/content/pm/PackageManager;)Ljava/lang/CharSequence;

    move-result-object v4

    invoke-interface {v4}, Ljava/lang/CharSequence;->toString()Ljava/lang/String;

    move-result-object v4

    new-instance v6, Ljava/lang/StringBuilder;

    invoke-direct {v6}, Ljava/lang/StringBuilder;-><init>()V

    invoke-virtual {v6, v4}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v4

    const-string v6, " \u00b7 "

    invoke-virtual {v4, v6}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v4

    invoke-virtual {v4, v5}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v4

    invoke-virtual {v4}, Ljava/lang/StringBuilder;->toString()Ljava/lang/String;

    move-result-object v4

    invoke-interface {v3, v4, v5}, Ljava/util/Map;->put(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;

    goto :goto_0

    .line 122
    :cond_1
    invoke-interface {v3}, Ljava/util/Map;->keySet()Ljava/util/Set;

    move-result-object v0

    new-array v1, v2, [Ljava/lang/String;

    invoke-interface {v0, v1}, Ljava/util/Set;->toArray([Ljava/lang/Object;)[Ljava/lang/Object;

    move-result-object v0

    check-cast v0, [Ljava/lang/String;

    invoke-interface {v3}, Ljava/util/Map;->values()Ljava/util/Collection;

    move-result-object v1

    new-array v3, v2, [Ljava/lang/String;

    invoke-interface {v1, v3}, Ljava/util/Collection;->toArray([Ljava/lang/Object;)[Ljava/lang/Object;

    move-result-object v1

    check-cast v1, [Ljava/lang/String;

    new-instance v3, Ljava/util/HashSet;

    iget-object v4, p0, Lorg/garlic/client/MainActivity;->prefs:Landroid/content/SharedPreferences;

    const-string v5, "bypass_packages"

    invoke-static {}, Ljava/util/Collections;->emptySet()Ljava/util/Set;

    move-result-object v6

    invoke-interface {v4, v5, v6}, Landroid/content/SharedPreferences;->getStringSet(Ljava/lang/String;Ljava/util/Set;)Ljava/util/Set;

    move-result-object v4

    invoke-direct {v3, v4}, Ljava/util/HashSet;-><init>(Ljava/util/Collection;)V

    array-length v4, v0

    new-array v4, v4, [Z

    :goto_1
    array-length v5, v0

    if-ge v2, v5, :cond_2

    aget-object v5, v1, v2

    invoke-interface {v3, v5}, Ljava/util/Set;->contains(Ljava/lang/Object;)Z

    move-result v5

    aput-boolean v5, v4, v2

    add-int/lit8 v2, v2, 0x1

    goto :goto_1

    .line 123
    :cond_2
    new-instance v2, Landroid/app/AlertDialog$Builder;

    invoke-direct {v2, p0}, Landroid/app/AlertDialog$Builder;-><init>(Landroid/content/Context;)V

    const-string v5, "\u0420\u0430\u0431\u043e\u0442\u0430\u044e\u0442 \u043d\u0430\u043f\u0440\u044f\u043c\u0443\u044e, \u0432\u043d\u0435 Garlic"

    invoke-virtual {v2, v5}, Landroid/app/AlertDialog$Builder;->setTitle(Ljava/lang/CharSequence;)Landroid/app/AlertDialog$Builder;

    move-result-object v2

    new-instance v5, Lorg/garlic/client/MainActivity$49;

    invoke-direct {v5, v3, v1}, Lorg/garlic/client/MainActivity$49;-><init>(Ljava/util/Set;[Ljava/lang/String;)V

    invoke-virtual {v2, v0, v4, v5}, Landroid/app/AlertDialog$Builder;->setMultiChoiceItems([Ljava/lang/CharSequence;[ZLandroid/content/DialogInterface$OnMultiChoiceClickListener;)Landroid/app/AlertDialog$Builder;

    move-result-object v0

    const-string v1, "\u041e\u0442\u043c\u0435\u043d\u0430"

    const/4 v2, 0x0

    invoke-virtual {v0, v1, v2}, Landroid/app/AlertDialog$Builder;->setNegativeButton(Ljava/lang/CharSequence;Landroid/content/DialogInterface$OnClickListener;)Landroid/app/AlertDialog$Builder;

    move-result-object v0

    new-instance v1, Lorg/garlic/client/MainActivity$50;

    invoke-direct {v1, p0, v3}, Lorg/garlic/client/MainActivity$50;-><init>(Lorg/garlic/client/MainActivity;Ljava/util/Set;)V

    const-string v2, "\u0421\u043e\u0445\u0440\u0430\u043d\u0438\u0442\u044c"

    invoke-virtual {v0, v2, v1}, Landroid/app/AlertDialog$Builder;->setPositiveButton(Ljava/lang/CharSequence;Landroid/content/DialogInterface$OnClickListener;)Landroid/app/AlertDialog$Builder;

    move-result-object v0

    invoke-virtual {v0}, Landroid/app/AlertDialog$Builder;->show()Landroid/app/AlertDialog;

    .line 124
    return-void
.end method

.method private dir()Ljava/io/File;
    .locals 3

    .line 60
    new-instance v0, Ljava/io/File;

    invoke-virtual {p0}, Lorg/garlic/client/MainActivity;->getFilesDir()Ljava/io/File;

    move-result-object v1

    const-string v2, "garlic"

    invoke-direct {v0, v1, v2}, Ljava/io/File;-><init>(Ljava/io/File;Ljava/lang/String;)V

    invoke-virtual {v0}, Ljava/io/File;->mkdirs()Z

    return-object v0
.end method

.method private dp(I)I
    .locals 1

    .line 54
    int-to-float p1, p1

    invoke-virtual {p0}, Lorg/garlic/client/MainActivity;->getResources()Landroid/content/res/Resources;

    move-result-object v0

    invoke-virtual {v0}, Landroid/content/res/Resources;->getDisplayMetrics()Landroid/util/DisplayMetrics;

    move-result-object v0

    iget v0, v0, Landroid/util/DisplayMetrics;->density:F

    mul-float/2addr p1, v0

    invoke-static {p1}, Ljava/lang/Math;->round(F)I

    move-result p1

    return p1
.end method

.method private editServer(Ljava/lang/String;Ljava/lang/String;II)V
    .locals 7

    .line 151
    new-instance v0, Landroid/widget/LinearLayout;

    invoke-direct {v0, p0}, Landroid/widget/LinearLayout;-><init>(Landroid/content/Context;)V

    const/4 v1, 0x1

    invoke-virtual {v0, v1}, Landroid/widget/LinearLayout;->setOrientation(I)V

    const/16 v1, 0x18

    invoke-direct {p0, v1}, Lorg/garlic/client/MainActivity;->dp(I)I

    move-result v2

    invoke-direct {p0, v1}, Lorg/garlic/client/MainActivity;->dp(I)I

    move-result v1

    const/4 v3, 0x0

    invoke-virtual {v0, v2, v3, v1, v3}, Landroid/widget/LinearLayout;->setPadding(IIII)V

    iget v1, p0, Lorg/garlic/client/MainActivity;->muted:I

    const-string v2, "\u041c\u0430\u043a\u0441\u0438\u043c\u0443\u043c \u043f\u043e\u0442\u043e\u043a\u043e\u0432"

    const/16 v4, 0xe

    invoke-direct {p0, v2, v4, v1}, Lorg/garlic/client/MainActivity;->text(Ljava/lang/String;II)Landroid/widget/TextView;

    move-result-object v1

    invoke-virtual {v0, v1}, Landroid/widget/LinearLayout;->addView(Landroid/view/View;)V

    const-string v1, "1\u20134096"

    invoke-direct {p0, v1, v3}, Lorg/garlic/client/MainActivity;->input(Ljava/lang/String;Z)Landroid/widget/EditText;

    move-result-object v2

    const/4 v5, 0x2

    invoke-virtual {v2, v5}, Landroid/widget/EditText;->setInputType(I)V

    invoke-static {p3}, Ljava/lang/String;->valueOf(I)Ljava/lang/String;

    move-result-object p3

    invoke-virtual {v2, p3}, Landroid/widget/EditText;->setText(Ljava/lang/CharSequence;)V

    invoke-virtual {v0, v2}, Landroid/widget/LinearLayout;->addView(Landroid/view/View;)V

    const-string p3, "\u041c\u0430\u043a\u0441\u0438\u043c\u0443\u043c \u0440\u0430\u0431\u043e\u0447\u0438\u0445 \u043a\u0430\u043d\u0430\u043b\u043e\u0432"

    iget v6, p0, Lorg/garlic/client/MainActivity;->muted:I

    invoke-direct {p0, p3, v4, v6}, Lorg/garlic/client/MainActivity;->text(Ljava/lang/String;II)Landroid/widget/TextView;

    move-result-object p3

    invoke-virtual {v0, p3}, Landroid/widget/LinearLayout;->addView(Landroid/view/View;)V

    invoke-direct {p0, v1, v3}, Lorg/garlic/client/MainActivity;->input(Ljava/lang/String;Z)Landroid/widget/EditText;

    move-result-object p3

    invoke-virtual {p3, v5}, Landroid/widget/EditText;->setInputType(I)V

    invoke-static {p4}, Ljava/lang/String;->valueOf(I)Ljava/lang/String;

    move-result-object p4

    invoke-virtual {p3, p4}, Landroid/widget/EditText;->setText(Ljava/lang/CharSequence;)V

    invoke-virtual {v0, p3}, Landroid/widget/LinearLayout;->addView(Landroid/view/View;)V

    .line 152
    new-instance p4, Landroid/app/AlertDialog$Builder;

    invoke-direct {p4, p0}, Landroid/app/AlertDialog$Builder;-><init>(Landroid/content/Context;)V

    invoke-virtual {p4, p2}, Landroid/app/AlertDialog$Builder;->setTitle(Ljava/lang/CharSequence;)Landroid/app/AlertDialog$Builder;

    move-result-object p2

    invoke-virtual {p2, v0}, Landroid/app/AlertDialog$Builder;->setView(Landroid/view/View;)Landroid/app/AlertDialog$Builder;

    move-result-object p2

    const-string p4, "\u041e\u0442\u043c\u0435\u043d\u0430"

    const/4 v0, 0x0

    invoke-virtual {p2, p4, v0}, Landroid/app/AlertDialog$Builder;->setNegativeButton(Ljava/lang/CharSequence;Landroid/content/DialogInterface$OnClickListener;)Landroid/app/AlertDialog$Builder;

    move-result-object p2

    new-instance p4, Lorg/garlic/client/MainActivity$48;

    invoke-direct {p4, p0, p1, v2, p3}, Lorg/garlic/client/MainActivity$48;-><init>(Lorg/garlic/client/MainActivity;Ljava/lang/String;Landroid/widget/EditText;Landroid/widget/EditText;)V

    const-string p1, "\u0421\u043e\u0445\u0440\u0430\u043d\u0438\u0442\u044c"

    invoke-virtual {p2, p1, p4}, Landroid/app/AlertDialog$Builder;->setPositiveButton(Ljava/lang/CharSequence;Landroid/content/DialogInterface$OnClickListener;)Landroid/app/AlertDialog$Builder;

    move-result-object p1

    .line 154
    invoke-virtual {p1}, Landroid/app/AlertDialog$Builder;->show()Landroid/app/AlertDialog;

    .line 155
    return-void
.end method

.method private editUser()V
    .locals 1

    .line 171
    const/4 v0, 0x0

    invoke-direct {p0, v0}, Lorg/garlic/client/MainActivity;->subscriptionDialog(Lorg/json/JSONObject;)V

    return-void
.end method

.method private input(Ljava/lang/String;Z)Landroid/widget/EditText;
    .locals 7

    .line 59
    new-instance v0, Landroid/widget/EditText;

    invoke-direct {v0, p0}, Landroid/widget/EditText;-><init>(Landroid/content/Context;)V

    const/4 v1, 0x1

    invoke-virtual {v0, v1}, Landroid/widget/EditText;->setSingleLine(Z)V

    iget v2, p0, Lorg/garlic/client/MainActivity;->ink:I

    invoke-virtual {v0, v2}, Landroid/widget/EditText;->setTextColor(I)V

    iget v2, p0, Lorg/garlic/client/MainActivity;->muted:I

    invoke-virtual {v0, v2}, Landroid/widget/EditText;->setHintTextColor(I)V

    invoke-virtual {v0, p1}, Landroid/widget/EditText;->setHint(Ljava/lang/CharSequence;)V

    const/high16 p1, 0x41800000    # 16.0f

    invoke-virtual {v0, p1}, Landroid/widget/EditText;->setTextSize(F)V

    const/16 p1, 0x10

    invoke-direct {p0, p1}, Lorg/garlic/client/MainActivity;->dp(I)I

    move-result v2

    const/16 v3, 0xe

    invoke-direct {p0, v3}, Lorg/garlic/client/MainActivity;->dp(I)I

    move-result v4

    invoke-direct {p0, p1}, Lorg/garlic/client/MainActivity;->dp(I)I

    move-result v5

    invoke-direct {p0, v3}, Lorg/garlic/client/MainActivity;->dp(I)I

    move-result v6

    invoke-virtual {v0, v2, v4, v5, v6}, Landroid/widget/EditText;->setPadding(IIII)V

    iget v2, p0, Lorg/garlic/client/MainActivity;->surface:I

    invoke-direct {p0, v2, v3}, Lorg/garlic/client/MainActivity;->shape(II)Landroid/graphics/drawable/GradientDrawable;

    move-result-object v2

    invoke-virtual {v0, v2}, Landroid/widget/EditText;->setBackground(Landroid/graphics/drawable/Drawable;)V

    if-eqz p2, :cond_0

    const/16 p1, 0x80

    :cond_0
    or-int/2addr p1, v1

    invoke-virtual {v0, p1}, Landroid/widget/EditText;->setInputType(I)V

    new-instance p1, Landroid/widget/LinearLayout$LayoutParams;

    const/16 p2, 0x38

    invoke-direct {p0, p2}, Lorg/garlic/client/MainActivity;->dp(I)I

    move-result p2

    const/4 v1, -0x1

    invoke-direct {p1, v1, p2}, Landroid/widget/LinearLayout$LayoutParams;-><init>(II)V

    const/4 p2, 0x6

    invoke-direct {p0, p2}, Lorg/garlic/client/MainActivity;->dp(I)I

    move-result v1

    invoke-direct {p0, p2}, Lorg/garlic/client/MainActivity;->dp(I)I

    move-result p2

    const/4 v2, 0x0

    invoke-virtual {p1, v2, v1, v2, p2}, Landroid/widget/LinearLayout$LayoutParams;->setMargins(IIII)V

    invoke-virtual {v0, p1}, Landroid/widget/EditText;->setLayoutParams(Landroid/view/ViewGroup$LayoutParams;)V

    return-object v0
.end method

.method static synthetic lambda$chooseApps$23(Ljava/util/Set;[Ljava/lang/String;Landroid/content/DialogInterface;IZ)V
    .locals 0

    .line 123
    if-eqz p4, :cond_0

    aget-object p1, p1, p3

    invoke-interface {p0, p1}, Ljava/util/Set;->add(Ljava/lang/Object;)Z

    goto :goto_0

    :cond_0
    aget-object p1, p1, p3

    invoke-interface {p0, p1}, Ljava/util/Set;->remove(Ljava/lang/Object;)Z

    :goto_0
    return-void
.end method

.method static synthetic lambda$subscriptionDialog$42(Ljava/util/Calendar;Landroid/widget/Button;Landroid/widget/DatePicker;III)V
    .locals 7

    .line 165
    const/16 v5, 0x3b

    const/16 v6, 0x3b

    const/16 v4, 0x17

    move-object v0, p0

    move v1, p3

    move v2, p4

    move v3, p5

    invoke-virtual/range {v0 .. v6}, Ljava/util/Calendar;->set(IIIIII)V

    new-instance p0, Ljava/util/Locale;

    const-string p2, "ru"

    const-string p3, "RU"

    invoke-direct {p0, p2, p3}, Ljava/util/Locale;-><init>(Ljava/lang/String;Ljava/lang/String;)V

    const/4 p2, 0x1

    invoke-static {p2, p0}, Ljava/text/DateFormat;->getDateInstance(ILjava/util/Locale;)Ljava/text/DateFormat;

    move-result-object p0

    invoke-virtual {v0}, Ljava/util/Calendar;->getTime()Ljava/util/Date;

    move-result-object p2

    invoke-virtual {p0, p2}, Ljava/text/DateFormat;->format(Ljava/util/Date;)Ljava/lang/String;

    move-result-object p0

    new-instance p2, Ljava/lang/StringBuilder;

    invoke-direct {p2}, Ljava/lang/StringBuilder;-><init>()V

    const-string p3, "\u0410\u043a\u0442\u0438\u0432\u043d\u0430 \u0434\u043e "

    invoke-virtual {p2, p3}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object p2

    invoke-virtual {p2, p0}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object p0

    invoke-virtual {p0}, Ljava/lang/StringBuilder;->toString()Ljava/lang/String;

    move-result-object p0

    invoke-virtual {p1, p0}, Landroid/widget/Button;->setText(Ljava/lang/CharSequence;)V

    return-void
.end method

.method static synthetic lambda$subscriptionDialog$44(Landroid/widget/Button;Landroid/widget/CompoundButton;Z)V
    .locals 0

    .line 166
    xor-int/lit8 p1, p2, 0x1

    invoke-virtual {p0, p1}, Landroid/widget/Button;->setEnabled(Z)V

    return-void
.end method

.method private login(Landroid/widget/Button;)V
    .locals 8

    .line 86
    iget-object v0, p0, Lorg/garlic/client/MainActivity;->server:Landroid/widget/EditText;

    invoke-virtual {v0}, Landroid/widget/EditText;->getText()Landroid/text/Editable;

    move-result-object v0

    invoke-virtual {v0}, Ljava/lang/Object;->toString()Ljava/lang/String;

    move-result-object v0

    invoke-virtual {v0}, Ljava/lang/String;->trim()Ljava/lang/String;

    move-result-object v0

    iget-object v1, p0, Lorg/garlic/client/MainActivity;->key:Landroid/widget/EditText;

    invoke-virtual {v1}, Landroid/widget/EditText;->getText()Landroid/text/Editable;

    move-result-object v1

    invoke-virtual {v1}, Ljava/lang/Object;->toString()Ljava/lang/String;

    move-result-object v1

    invoke-virtual {v1}, Ljava/lang/String;->trim()Ljava/lang/String;

    move-result-object v5

    invoke-virtual {v0}, Ljava/lang/String;->isEmpty()Z

    move-result v1

    if-nez v1, :cond_3

    invoke-virtual {v5}, Ljava/lang/String;->isEmpty()Z

    move-result v1

    if-eqz v1, :cond_0

    goto :goto_1

    .line 87
    :cond_0
    const-string v1, "://"

    invoke-virtual {v0, v1}, Ljava/lang/String;->contains(Ljava/lang/CharSequence;)Z

    move-result v1

    if-nez v1, :cond_2

    const-string v1, ":"

    invoke-virtual {v0, v1}, Ljava/lang/String;->contains(Ljava/lang/CharSequence;)Z

    move-result v1

    if-eqz v1, :cond_1

    const-string v1, ""

    goto :goto_0

    :cond_1
    const-string v1, ":8443"

    :goto_0
    new-instance v2, Ljava/lang/StringBuilder;

    invoke-direct {v2}, Ljava/lang/StringBuilder;-><init>()V

    const-string v3, "https://"

    invoke-virtual {v2, v3}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v2

    invoke-virtual {v2, v0}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v0

    invoke-virtual {v0, v1}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v0

    invoke-virtual {v0}, Ljava/lang/StringBuilder;->toString()Ljava/lang/String;

    move-result-object v0

    :cond_2
    move-object v4, v0

    iget v0, p0, Lorg/garlic/client/MainActivity;->generation:I

    add-int/lit8 v6, v0, 0x1

    iput v6, p0, Lorg/garlic/client/MainActivity;->generation:I

    const/4 v0, 0x0

    invoke-virtual {p1, v0}, Landroid/widget/Button;->setEnabled(Z)V

    iget-object v0, p0, Lorg/garlic/client/MainActivity;->message:Landroid/widget/TextView;

    const-string v1, "\u041f\u0440\u043e\u0432\u0435\u0440\u044f\u0435\u043c \u043a\u043b\u044e\u0447 \u0438 \u043f\u043e\u043b\u0443\u0447\u0430\u0435\u043c \u0441\u0435\u0442\u044c\u2026"

    invoke-virtual {v0, v1}, Landroid/widget/TextView;->setText(Ljava/lang/CharSequence;)V

    .line 88
    iget-object v0, p0, Lorg/garlic/client/MainActivity;->worker:Ljava/util/concurrent/ExecutorService;

    new-instance v2, Lorg/garlic/client/MainActivity$45;

    move-object v3, p0

    move-object v7, p1

    invoke-direct/range {v2 .. v7}, Lorg/garlic/client/MainActivity$45;-><init>(Lorg/garlic/client/MainActivity;Ljava/lang/String;Ljava/lang/String;ILandroid/widget/Button;)V

    invoke-interface {v0, v2}, Ljava/util/concurrent/ExecutorService;->execute(Ljava/lang/Runnable;)V

    .line 89
    return-void

    .line 86
    :cond_3
    :goto_1
    iget-object p1, p0, Lorg/garlic/client/MainActivity;->message:Landroid/widget/TextView;

    const-string v0, "\u0412\u0432\u0435\u0434\u0438\u0442\u0435 \u0441\u0435\u0440\u0432\u0435\u0440 \u0438 \u043a\u043b\u044e\u0447"

    invoke-virtual {p1, v0}, Landroid/widget/TextView;->setText(Ljava/lang/CharSequence;)V

    return-void
.end method

.method private logout()V
    .locals 2

    .line 117
    iget v0, p0, Lorg/garlic/client/MainActivity;->generation:I

    add-int/lit8 v0, v0, 0x1

    iput v0, p0, Lorg/garlic/client/MainActivity;->generation:I

    const/4 v0, 0x0

    iput-boolean v0, p0, Lorg/garlic/client/MainActivity;->loggedIn:Z

    iput-boolean v0, p0, Lorg/garlic/client/MainActivity;->admin:Z

    iput-boolean v0, p0, Lorg/garlic/client/MainActivity;->roleVerified:Z

    invoke-direct {p0}, Lorg/garlic/client/MainActivity;->stopVpn()V

    iget-object v0, p0, Lorg/garlic/client/MainActivity;->worker:Ljava/util/concurrent/ExecutorService;

    new-instance v1, Lorg/garlic/client/MainActivity$46;

    invoke-direct {v1, p0}, Lorg/garlic/client/MainActivity$46;-><init>(Lorg/garlic/client/MainActivity;)V

    invoke-interface {v0, v1}, Ljava/util/concurrent/ExecutorService;->execute(Ljava/lang/Runnable;)V

    return-void
.end method

.method private manage(Lorg/json/JSONObject;)V
    .locals 3

    .line 182
    iget v0, p0, Lorg/garlic/client/MainActivity;->generation:I

    iget-object v1, p0, Lorg/garlic/client/MainActivity;->message:Landroid/widget/TextView;

    const-string v2, "\u0421\u043e\u0445\u0440\u0430\u043d\u044f\u0435\u043c\u2026"

    invoke-virtual {v1, v2}, Landroid/widget/TextView;->setText(Ljava/lang/CharSequence;)V

    iget-object v1, p0, Lorg/garlic/client/MainActivity;->worker:Ljava/util/concurrent/ExecutorService;

    new-instance v2, Lorg/garlic/client/MainActivity$51;

    invoke-direct {v2, p0, p1, v0}, Lorg/garlic/client/MainActivity$51;-><init>(Lorg/garlic/client/MainActivity;Lorg/json/JSONObject;I)V

    invoke-interface {v1, v2}, Ljava/util/concurrent/ExecutorService;->execute(Ljava/lang/Runnable;)V

    return-void
.end method

.method private origins()Ljava/lang/String;
    .locals 5

    .line 61
    iget-object v0, p0, Lorg/garlic/client/MainActivity;->prefs:Landroid/content/SharedPreferences;

    const-string v1, "bootstrap"

    const-string v2, ""

    invoke-interface {v0, v1, v2}, Landroid/content/SharedPreferences;->getString(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;

    move-result-object v0

    iget-object v1, p0, Lorg/garlic/client/MainActivity;->prefs:Landroid/content/SharedPreferences;

    const-string v3, "direct_urls"

    invoke-interface {v1, v3, v2}, Landroid/content/SharedPreferences;->getString(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;

    move-result-object v1

    iget-object v3, p0, Lorg/garlic/client/MainActivity;->prefs:Landroid/content/SharedPreferences;

    const-string v4, "ru_urls"

    invoke-interface {v3, v4, v2}, Landroid/content/SharedPreferences;->getString(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;

    move-result-object v2

    new-instance v3, Ljava/lang/StringBuilder;

    invoke-direct {v3}, Ljava/lang/StringBuilder;-><init>()V

    invoke-virtual {v3, v0}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v0

    const-string v3, ","

    invoke-virtual {v0, v3}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v0

    invoke-virtual {v0, v1}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v0

    invoke-virtual {v0, v3}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v0

    invoke-virtual {v0, v2}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v0

    invoke-virtual {v0}, Ljava/lang/StringBuilder;->toString()Ljava/lang/String;

    move-result-object v0

    return-object v0
.end method

.method private refreshAccount()V
    .locals 3

    .line 90
    sget-object v0, Lorg/garlic/client/GarlicBridge;->loadError:Ljava/lang/String;

    if-nez v0, :cond_1

    invoke-direct {p0}, Lorg/garlic/client/MainActivity;->origins()Ljava/lang/String;

    move-result-object v0

    const-string v1, ","

    invoke-virtual {v0, v1}, Ljava/lang/String;->equals(Ljava/lang/Object;)Z

    move-result v0

    if-eqz v0, :cond_0

    goto :goto_0

    :cond_0
    iget v0, p0, Lorg/garlic/client/MainActivity;->generation:I

    iget-object v1, p0, Lorg/garlic/client/MainActivity;->worker:Ljava/util/concurrent/ExecutorService;

    new-instance v2, Lorg/garlic/client/MainActivity$52;

    invoke-direct {v2, p0, v0}, Lorg/garlic/client/MainActivity$52;-><init>(Lorg/garlic/client/MainActivity;I)V

    invoke-interface {v1, v2}, Ljava/util/concurrent/ExecutorService;->execute(Ljava/lang/Runnable;)V

    :cond_1
    :goto_0
    return-void
.end method

.method private routeName()Ljava/lang/String;
    .locals 3

    .line 108
    iget-object v0, p0, Lorg/garlic/client/MainActivity;->prefs:Landroid/content/SharedPreferences;

    const-string v1, "mode"

    const-string v2, "auto"

    invoke-interface {v0, v1, v2}, Landroid/content/SharedPreferences;->getString(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;

    move-result-object v0

    const-string v1, "ru"

    invoke-virtual {v1, v0}, Ljava/lang/String;->equals(Ljava/lang/Object;)Z

    move-result v1

    if-eqz v1, :cond_0

    const-string v0, "\u0447\u0435\u0440\u0435\u0437 \u0420\u043e\u0441\u0441\u0438\u044e"

    goto :goto_0

    :cond_0
    const-string v1, "direct"

    invoke-virtual {v1, v0}, Ljava/lang/String;->equals(Ljava/lang/Object;)Z

    move-result v0

    if-eqz v0, :cond_1

    const-string v0, "\u043d\u0430\u043f\u0440\u044f\u043c\u0443\u044e"

    goto :goto_0

    :cond_1
    const-string v0, "\u0430\u0432\u0442\u043e\u043c\u0430\u0442\u0438\u0447\u0435\u0441\u043a\u0438"

    :goto_0
    return-object v0
.end method

.method private shape(II)Landroid/graphics/drawable/GradientDrawable;
    .locals 1

    .line 55
    new-instance v0, Landroid/graphics/drawable/GradientDrawable;

    invoke-direct {v0}, Landroid/graphics/drawable/GradientDrawable;-><init>()V

    invoke-virtual {v0, p1}, Landroid/graphics/drawable/GradientDrawable;->setColor(I)V

    invoke-direct {p0, p2}, Lorg/garlic/client/MainActivity;->dp(I)I

    move-result p1

    int-to-float p1, p1

    invoke-virtual {v0, p1}, Landroid/graphics/drawable/GradientDrawable;->setCornerRadius(F)V

    return-object v0
.end method

.method private shell(Ljava/lang/String;)V
    .locals 8

    .line 64
    const/4 v0, 0x0

    iput-object v0, p0, Lorg/garlic/client/MainActivity;->power:Landroid/widget/Button;

    iput-object v0, p0, Lorg/garlic/client/MainActivity;->routeInfo:Landroid/widget/TextView;

    iput-object v0, p0, Lorg/garlic/client/MainActivity;->statsInfo:Landroid/widget/TextView;

    iput-object v0, p0, Lorg/garlic/client/MainActivity;->subscriptionInfo:Landroid/widget/TextView;

    new-instance v0, Landroid/widget/LinearLayout;

    invoke-direct {v0, p0}, Landroid/widget/LinearLayout;-><init>(Landroid/content/Context;)V

    iput-object v0, p0, Lorg/garlic/client/MainActivity;->page:Landroid/widget/LinearLayout;

    iget-object v0, p0, Lorg/garlic/client/MainActivity;->page:Landroid/widget/LinearLayout;

    const/4 v1, 0x1

    invoke-virtual {v0, v1}, Landroid/widget/LinearLayout;->setOrientation(I)V

    iget-object v0, p0, Lorg/garlic/client/MainActivity;->page:Landroid/widget/LinearLayout;

    iget v2, p0, Lorg/garlic/client/MainActivity;->bg:I

    invoke-virtual {v0, v2}, Landroid/widget/LinearLayout;->setBackgroundColor(I)V

    iget-object v0, p0, Lorg/garlic/client/MainActivity;->page:Landroid/widget/LinearLayout;

    const/16 v2, 0x18

    invoke-direct {p0, v2}, Lorg/garlic/client/MainActivity;->dp(I)I

    move-result v3

    const/16 v4, 0xc

    invoke-direct {p0, v4}, Lorg/garlic/client/MainActivity;->dp(I)I

    move-result v4

    invoke-direct {p0, v2}, Lorg/garlic/client/MainActivity;->dp(I)I

    move-result v5

    const/16 v6, 0x14

    invoke-direct {p0, v6}, Lorg/garlic/client/MainActivity;->dp(I)I

    move-result v6

    invoke-virtual {v0, v3, v4, v5, v6}, Landroid/widget/LinearLayout;->setPadding(IIII)V

    .line 65
    new-instance v0, Landroid/widget/LinearLayout;

    invoke-direct {v0, p0}, Landroid/widget/LinearLayout;-><init>(Landroid/content/Context;)V

    iput-object v0, p0, Lorg/garlic/client/MainActivity;->header:Landroid/widget/LinearLayout;

    iget-object v0, p0, Lorg/garlic/client/MainActivity;->header:Landroid/widget/LinearLayout;

    const/16 v3, 0x10

    invoke-virtual {v0, v3}, Landroid/widget/LinearLayout;->setGravity(I)V

    .line 66
    new-instance v0, Landroid/widget/ImageView;

    invoke-direct {v0, p0}, Landroid/widget/ImageView;-><init>(Landroid/content/Context;)V

    invoke-virtual {p0}, Lorg/garlic/client/MainActivity;->getResources()Landroid/content/res/Resources;

    move-result-object v3

    const-string v4, "drawable"

    invoke-virtual {p0}, Lorg/garlic/client/MainActivity;->getPackageName()Ljava/lang/String;

    move-result-object v5

    const-string v6, "ic_garlic_notification"

    invoke-virtual {v3, v6, v4, v5}, Landroid/content/res/Resources;->getIdentifier(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)I

    move-result v3

    invoke-virtual {v0, v3}, Landroid/widget/ImageView;->setImageResource(I)V

    iget-object v3, p0, Lorg/garlic/client/MainActivity;->header:Landroid/widget/LinearLayout;

    new-instance v4, Landroid/widget/LinearLayout$LayoutParams;

    const/16 v5, 0x24

    invoke-direct {p0, v5}, Lorg/garlic/client/MainActivity;->dp(I)I

    move-result v5

    const/16 v6, 0x28

    invoke-direct {p0, v6}, Lorg/garlic/client/MainActivity;->dp(I)I

    move-result v6

    invoke-direct {v4, v5, v6}, Landroid/widget/LinearLayout$LayoutParams;-><init>(II)V

    invoke-virtual {v3, v0, v4}, Landroid/widget/LinearLayout;->addView(Landroid/view/View;Landroid/view/ViewGroup$LayoutParams;)V

    .line 67
    const-string v0, "  Garlic"

    iget v3, p0, Lorg/garlic/client/MainActivity;->ink:I

    invoke-direct {p0, v0, v2, v3}, Lorg/garlic/client/MainActivity;->text(Ljava/lang/String;II)Landroid/widget/TextView;

    move-result-object v0

    sget-object v2, Landroid/graphics/Typeface;->DEFAULT:Landroid/graphics/Typeface;

    invoke-virtual {v0, v2, v1}, Landroid/widget/TextView;->setTypeface(Landroid/graphics/Typeface;I)V

    iget-object v2, p0, Lorg/garlic/client/MainActivity;->header:Landroid/widget/LinearLayout;

    new-instance v3, Landroid/widget/LinearLayout$LayoutParams;

    const/16 v4, 0x40

    invoke-direct {p0, v4}, Lorg/garlic/client/MainActivity;->dp(I)I

    move-result v4

    const/4 v5, 0x0

    const/high16 v6, 0x3f800000    # 1.0f

    invoke-direct {v3, v5, v4, v6}, Landroid/widget/LinearLayout$LayoutParams;-><init>(IIF)V

    invoke-virtual {v2, v0, v3}, Landroid/widget/LinearLayout;->addView(Landroid/view/View;Landroid/view/ViewGroup$LayoutParams;)V

    .line 68
    iget-boolean v0, p0, Lorg/garlic/client/MainActivity;->loggedIn:Z

    if-eqz v0, :cond_0

    iget-boolean v0, p0, Lorg/garlic/client/MainActivity;->admin:Z

    if-eqz v0, :cond_0

    const-string v0, "\u0423\u043f\u0440\u0430\u0432\u043b\u0435\u043d\u0438\u0435"

    invoke-direct {p0, v0, v5}, Lorg/garlic/client/MainActivity;->button(Ljava/lang/String;Z)Landroid/widget/Button;

    move-result-object v0

    const/high16 v2, 0x41500000    # 13.0f

    invoke-virtual {v0, v2}, Landroid/widget/Button;->setTextSize(F)V

    new-instance v2, Lorg/garlic/client/MainActivity$17;

    invoke-direct {v2, p0}, Lorg/garlic/client/MainActivity$17;-><init>(Lorg/garlic/client/MainActivity;)V

    invoke-virtual {v0, v2}, Landroid/widget/Button;->setOnClickListener(Landroid/view/View$OnClickListener;)V

    iget-object v2, p0, Lorg/garlic/client/MainActivity;->header:Landroid/widget/LinearLayout;

    new-instance v3, Landroid/widget/LinearLayout$LayoutParams;

    const/16 v4, 0x91

    invoke-direct {p0, v4}, Lorg/garlic/client/MainActivity;->dp(I)I

    move-result v4

    const/16 v7, 0x2c

    invoke-direct {p0, v7}, Lorg/garlic/client/MainActivity;->dp(I)I

    move-result v7

    invoke-direct {v3, v4, v7}, Landroid/widget/LinearLayout$LayoutParams;-><init>(II)V

    invoke-virtual {v2, v0, v3}, Landroid/widget/LinearLayout;->addView(Landroid/view/View;Landroid/view/ViewGroup$LayoutParams;)V

    .line 69
    :cond_0
    iget-object v0, p0, Lorg/garlic/client/MainActivity;->page:Landroid/widget/LinearLayout;

    iget-object v2, p0, Lorg/garlic/client/MainActivity;->header:Landroid/widget/LinearLayout;

    invoke-virtual {v0, v2}, Landroid/widget/LinearLayout;->addView(Landroid/view/View;)V

    .line 70
    new-instance v0, Landroid/widget/ScrollView;

    invoke-direct {v0, p0}, Landroid/widget/ScrollView;-><init>(Landroid/content/Context;)V

    invoke-virtual {v0, v1}, Landroid/widget/ScrollView;->setFillViewport(Z)V

    new-instance v2, Landroid/widget/LinearLayout;

    invoke-direct {v2, p0}, Landroid/widget/LinearLayout;-><init>(Landroid/content/Context;)V

    iput-object v2, p0, Lorg/garlic/client/MainActivity;->content:Landroid/widget/LinearLayout;

    iget-object v2, p0, Lorg/garlic/client/MainActivity;->content:Landroid/widget/LinearLayout;

    invoke-virtual {v2, v1}, Landroid/widget/LinearLayout;->setOrientation(I)V

    iget-object v1, p0, Lorg/garlic/client/MainActivity;->content:Landroid/widget/LinearLayout;

    invoke-virtual {v0, v1}, Landroid/widget/ScrollView;->addView(Landroid/view/View;)V

    iget-object v1, p0, Lorg/garlic/client/MainActivity;->page:Landroid/widget/LinearLayout;

    new-instance v2, Landroid/widget/LinearLayout$LayoutParams;

    const/4 v3, -0x1

    invoke-direct {v2, v3, v5, v6}, Landroid/widget/LinearLayout$LayoutParams;-><init>(IIF)V

    invoke-virtual {v1, v0, v2}, Landroid/widget/LinearLayout;->addView(Landroid/view/View;Landroid/view/ViewGroup$LayoutParams;)V

    .line 71
    invoke-virtual {p1}, Ljava/lang/String;->isEmpty()Z

    move-result v0

    const/16 v1, 0xe

    if-nez v0, :cond_1

    iget-object v0, p0, Lorg/garlic/client/MainActivity;->content:Landroid/widget/LinearLayout;

    iget v2, p0, Lorg/garlic/client/MainActivity;->muted:I

    invoke-direct {p0, p1, v1, v2}, Lorg/garlic/client/MainActivity;->text(Ljava/lang/String;II)Landroid/widget/TextView;

    move-result-object p1

    invoke-virtual {v0, p1}, Landroid/widget/LinearLayout;->addView(Landroid/view/View;)V

    .line 72
    :cond_1
    const-string p1, ""

    iget v0, p0, Lorg/garlic/client/MainActivity;->muted:I

    invoke-direct {p0, p1, v1, v0}, Lorg/garlic/client/MainActivity;->text(Ljava/lang/String;II)Landroid/widget/TextView;

    move-result-object p1

    iput-object p1, p0, Lorg/garlic/client/MainActivity;->message:Landroid/widget/TextView;

    iget-object p1, p0, Lorg/garlic/client/MainActivity;->page:Landroid/widget/LinearLayout;

    invoke-virtual {p0, p1}, Lorg/garlic/client/MainActivity;->setContentView(Landroid/view/View;)V

    .line 73
    return-void
.end method

.method private showHome()V
    .locals 7

    .line 99
    iget-object v0, p0, Lorg/garlic/client/MainActivity;->account:Lorg/json/JSONObject;

    if-nez v0, :cond_0

    invoke-direct {p0}, Lorg/garlic/client/MainActivity;->showLogin()V

    return-void

    :cond_0
    iget-object v0, p0, Lorg/garlic/client/MainActivity;->account:Lorg/json/JSONObject;

    const-string v1, "principal"

    invoke-virtual {v0, v1}, Lorg/json/JSONObject;->optJSONObject(Ljava/lang/String;)Lorg/json/JSONObject;

    move-result-object v0

    iget-boolean v1, p0, Lorg/garlic/client/MainActivity;->roleVerified:Z

    const/4 v2, 0x1

    const/4 v3, 0x0

    if-eqz v1, :cond_1

    if-eqz v0, :cond_1

    const-string v1, "role"

    invoke-virtual {v0, v1}, Lorg/json/JSONObject;->optString(Ljava/lang/String;)Ljava/lang/String;

    move-result-object v0

    const-string v1, "admin"

    invoke-virtual {v1, v0}, Ljava/lang/String;->equals(Ljava/lang/Object;)Z

    move-result v0

    if-eqz v0, :cond_1

    move v0, v2

    goto :goto_0

    :cond_1
    move v0, v3

    :goto_0
    iput-boolean v0, p0, Lorg/garlic/client/MainActivity;->admin:Z

    iput-boolean v2, p0, Lorg/garlic/client/MainActivity;->loggedIn:Z

    const-string v0, ""

    invoke-direct {p0, v0}, Lorg/garlic/client/MainActivity;->shell(Ljava/lang/String;)V

    const/16 v1, 0x18

    invoke-direct {p0, v1}, Lorg/garlic/client/MainActivity;->space(I)V

    sget-boolean v1, Lorg/garlic/client/GarlicProxyService;->isRunning:Z

    if-eqz v1, :cond_2

    const-string v1, "\u041f\u043e\u0434\u043a\u043b\u044e\u0447\u0435\u043d\u043e"

    goto :goto_1

    :cond_2
    const-string v1, "\u041d\u0435 \u043f\u043e\u0434\u043a\u043b\u044e\u0447\u0435\u043d\u043e"

    :goto_1
    const/16 v4, 0x1c

    iget v5, p0, Lorg/garlic/client/MainActivity;->ink:I

    invoke-direct {p0, v1, v4, v5}, Lorg/garlic/client/MainActivity;->text(Ljava/lang/String;II)Landroid/widget/TextView;

    move-result-object v1

    iput-object v1, p0, Lorg/garlic/client/MainActivity;->connection:Landroid/widget/TextView;

    iget-object v1, p0, Lorg/garlic/client/MainActivity;->connection:Landroid/widget/TextView;

    const/16 v4, 0x11

    invoke-virtual {v1, v4}, Landroid/widget/TextView;->setGravity(I)V

    iget-object v1, p0, Lorg/garlic/client/MainActivity;->content:Landroid/widget/LinearLayout;

    iget-object v5, p0, Lorg/garlic/client/MainActivity;->connection:Landroid/widget/TextView;

    invoke-virtual {v1, v5}, Landroid/widget/LinearLayout;->addView(Landroid/view/View;)V

    invoke-direct {p0}, Lorg/garlic/client/MainActivity;->subscription()Ljava/lang/String;

    move-result-object v1

    const/16 v5, 0xf

    iget v6, p0, Lorg/garlic/client/MainActivity;->muted:I

    invoke-direct {p0, v1, v5, v6}, Lorg/garlic/client/MainActivity;->text(Ljava/lang/String;II)Landroid/widget/TextView;

    move-result-object v1

    iput-object v1, p0, Lorg/garlic/client/MainActivity;->subscriptionInfo:Landroid/widget/TextView;

    iget-object v1, p0, Lorg/garlic/client/MainActivity;->subscriptionInfo:Landroid/widget/TextView;

    invoke-virtual {v1, v4}, Landroid/widget/TextView;->setGravity(I)V

    iget-object v1, p0, Lorg/garlic/client/MainActivity;->content:Landroid/widget/LinearLayout;

    iget-object v5, p0, Lorg/garlic/client/MainActivity;->subscriptionInfo:Landroid/widget/TextView;

    invoke-virtual {v1, v5}, Landroid/widget/LinearLayout;->addView(Landroid/view/View;)V

    iget v1, p0, Lorg/garlic/client/MainActivity;->muted:I

    const/16 v5, 0xe

    invoke-direct {p0, v0, v5, v1}, Lorg/garlic/client/MainActivity;->text(Ljava/lang/String;II)Landroid/widget/TextView;

    move-result-object v1

    iput-object v1, p0, Lorg/garlic/client/MainActivity;->routeInfo:Landroid/widget/TextView;

    iget-object v1, p0, Lorg/garlic/client/MainActivity;->routeInfo:Landroid/widget/TextView;

    invoke-virtual {v1, v4}, Landroid/widget/TextView;->setGravity(I)V

    iget-object v1, p0, Lorg/garlic/client/MainActivity;->content:Landroid/widget/LinearLayout;

    iget-object v6, p0, Lorg/garlic/client/MainActivity;->routeInfo:Landroid/widget/TextView;

    invoke-virtual {v1, v6}, Landroid/widget/LinearLayout;->addView(Landroid/view/View;)V

    const/16 v1, 0xd

    iget v6, p0, Lorg/garlic/client/MainActivity;->muted:I

    invoke-direct {p0, v0, v1, v6}, Lorg/garlic/client/MainActivity;->text(Ljava/lang/String;II)Landroid/widget/TextView;

    move-result-object v0

    iput-object v0, p0, Lorg/garlic/client/MainActivity;->statsInfo:Landroid/widget/TextView;

    iget-object v0, p0, Lorg/garlic/client/MainActivity;->statsInfo:Landroid/widget/TextView;

    invoke-virtual {v0, v4}, Landroid/widget/TextView;->setGravity(I)V

    iget-object v0, p0, Lorg/garlic/client/MainActivity;->content:Landroid/widget/LinearLayout;

    iget-object v1, p0, Lorg/garlic/client/MainActivity;->statsInfo:Landroid/widget/TextView;

    invoke-virtual {v0, v1}, Landroid/widget/LinearLayout;->addView(Landroid/view/View;)V

    const/16 v0, 0x14

    invoke-direct {p0, v0}, Lorg/garlic/client/MainActivity;->space(I)V

    .line 100
    sget-boolean v1, Lorg/garlic/client/GarlicProxyService;->isRunning:Z

    if-eqz v1, :cond_3

    const-string v1, "\u041e\u0442\u043a\u043b\u044e\u0447\u0438\u0442\u044c"

    goto :goto_2

    :cond_3
    const-string v1, "\u041f\u043e\u0434\u043a\u043b\u044e\u0447\u0438\u0442\u044c"

    :goto_2
    invoke-direct {p0, v1, v2}, Lorg/garlic/client/MainActivity;->button(Ljava/lang/String;Z)Landroid/widget/Button;

    move-result-object v1

    iput-object v1, p0, Lorg/garlic/client/MainActivity;->power:Landroid/widget/Button;

    iget-object v1, p0, Lorg/garlic/client/MainActivity;->power:Landroid/widget/Button;

    new-instance v2, Landroid/widget/LinearLayout$LayoutParams;

    const/16 v4, 0x3a

    invoke-direct {p0, v4}, Lorg/garlic/client/MainActivity;->dp(I)I

    move-result v4

    const/4 v6, -0x1

    invoke-direct {v2, v6, v4}, Landroid/widget/LinearLayout$LayoutParams;-><init>(II)V

    invoke-virtual {v1, v2}, Landroid/widget/Button;->setLayoutParams(Landroid/view/ViewGroup$LayoutParams;)V

    iget-object v1, p0, Lorg/garlic/client/MainActivity;->power:Landroid/widget/Button;

    new-instance v2, Lorg/garlic/client/MainActivity$20;

    invoke-direct {v2, p0}, Lorg/garlic/client/MainActivity$20;-><init>(Lorg/garlic/client/MainActivity;)V

    invoke-virtual {v1, v2}, Landroid/widget/Button;->setOnClickListener(Landroid/view/View$OnClickListener;)V

    iget-object v1, p0, Lorg/garlic/client/MainActivity;->content:Landroid/widget/LinearLayout;

    iget-object v2, p0, Lorg/garlic/client/MainActivity;->power:Landroid/widget/Button;

    invoke-virtual {v1, v2}, Landroid/widget/LinearLayout;->addView(Landroid/view/View;)V

    const/16 v1, 0x10

    invoke-direct {p0, v1}, Lorg/garlic/client/MainActivity;->space(I)V

    .line 101
    invoke-direct {p0}, Lorg/garlic/client/MainActivity;->routeName()Ljava/lang/String;

    move-result-object v1

    new-instance v2, Ljava/lang/StringBuilder;

    invoke-direct {v2}, Ljava/lang/StringBuilder;-><init>()V

    const-string v4, "\u041c\u0430\u0440\u0448\u0440\u0443\u0442: "

    invoke-virtual {v2, v4}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v2

    invoke-virtual {v2, v1}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v1

    invoke-virtual {v1}, Ljava/lang/StringBuilder;->toString()Ljava/lang/String;

    move-result-object v1

    invoke-direct {p0, v1, v3}, Lorg/garlic/client/MainActivity;->button(Ljava/lang/String;Z)Landroid/widget/Button;

    move-result-object v1

    new-instance v2, Lorg/garlic/client/MainActivity$21;

    invoke-direct {v2, p0, v1}, Lorg/garlic/client/MainActivity$21;-><init>(Lorg/garlic/client/MainActivity;Landroid/widget/Button;)V

    invoke-virtual {v1, v2}, Landroid/widget/Button;->setOnClickListener(Landroid/view/View$OnClickListener;)V

    iget-object v2, p0, Lorg/garlic/client/MainActivity;->content:Landroid/widget/LinearLayout;

    invoke-virtual {v2, v1}, Landroid/widget/LinearLayout;->addView(Landroid/view/View;)V

    .line 102
    const-string v1, "\u041d\u0430\u0441\u0442\u0440\u043e\u0439\u043a\u0438 \u043f\u043e\u0434\u043a\u043b\u044e\u0447\u0435\u043d\u0438\u044f"

    invoke-direct {p0, v1, v3}, Lorg/garlic/client/MainActivity;->button(Ljava/lang/String;Z)Landroid/widget/Button;

    move-result-object v1

    new-instance v2, Lorg/garlic/client/MainActivity$22;

    invoke-direct {v2, p0}, Lorg/garlic/client/MainActivity$22;-><init>(Lorg/garlic/client/MainActivity;)V

    invoke-virtual {v1, v2}, Landroid/widget/Button;->setOnClickListener(Landroid/view/View$OnClickListener;)V

    iget-object v2, p0, Lorg/garlic/client/MainActivity;->content:Landroid/widget/LinearLayout;

    invoke-virtual {v2, v1}, Landroid/widget/LinearLayout;->addView(Landroid/view/View;)V

    invoke-static {p0, v2}, Lorg/garlic/client/ReleaseUpdate;->addButton(Landroid/app/Activity;Landroid/widget/LinearLayout;)V

    .line 103
    iget-object v1, p0, Lorg/garlic/client/MainActivity;->content:Landroid/widget/LinearLayout;

    iget-object v2, p0, Lorg/garlic/client/MainActivity;->message:Landroid/widget/TextView;

    invoke-virtual {v1, v2}, Landroid/widget/LinearLayout;->addView(Landroid/view/View;)V

    .line 104
    iget-boolean v1, p0, Lorg/garlic/client/MainActivity;->admin:Z

    if-nez v1, :cond_4

    const-string v1, "\u041f\u0435\u0440\u0435\u043a\u043b\u044e\u0447\u0438\u0442\u044c \u043d\u0430 \u044d\u0442\u043e \u0443\u0441\u0442\u0440\u043e\u0439\u0441\u0442\u0432\u043e"

    invoke-direct {p0, v1, v3}, Lorg/garlic/client/MainActivity;->button(Ljava/lang/String;Z)Landroid/widget/Button;

    move-result-object v1

    new-instance v2, Lorg/garlic/client/MainActivity$23;

    invoke-direct {v2, p0}, Lorg/garlic/client/MainActivity$23;-><init>(Lorg/garlic/client/MainActivity;)V

    invoke-virtual {v1, v2}, Landroid/widget/Button;->setOnClickListener(Landroid/view/View$OnClickListener;)V

    iget-object v2, p0, Lorg/garlic/client/MainActivity;->content:Landroid/widget/LinearLayout;

    invoke-virtual {v2, v1}, Landroid/widget/LinearLayout;->addView(Landroid/view/View;)V

    .line 105
    :cond_4
    iget-boolean v1, p0, Lorg/garlic/client/MainActivity;->admin:Z

    if-eqz v1, :cond_5

    invoke-direct {p0, v0}, Lorg/garlic/client/MainActivity;->space(I)V

    iget-object v1, p0, Lorg/garlic/client/MainActivity;->content:Landroid/widget/LinearLayout;

    const-string v2, "\u0410\u0434\u043c\u0438\u043d\u0438\u0441\u0442\u0440\u0438\u0440\u043e\u0432\u0430\u043d\u0438\u0435"

    iget v4, p0, Lorg/garlic/client/MainActivity;->muted:I

    invoke-direct {p0, v2, v5, v4}, Lorg/garlic/client/MainActivity;->text(Ljava/lang/String;II)Landroid/widget/TextView;

    move-result-object v2

    invoke-virtual {v1, v2}, Landroid/widget/LinearLayout;->addView(Landroid/view/View;)V

    const-string v1, "\u041f\u043e\u043b\u044c\u0437\u043e\u0432\u0430\u0442\u0435\u043b\u0438"

    invoke-direct {p0, v1, v3}, Lorg/garlic/client/MainActivity;->button(Ljava/lang/String;Z)Landroid/widget/Button;

    move-result-object v1

    new-instance v2, Lorg/garlic/client/MainActivity$24;

    invoke-direct {v2, p0}, Lorg/garlic/client/MainActivity$24;-><init>(Lorg/garlic/client/MainActivity;)V

    invoke-virtual {v1, v2}, Landroid/widget/Button;->setOnClickListener(Landroid/view/View$OnClickListener;)V

    iget-object v2, p0, Lorg/garlic/client/MainActivity;->content:Landroid/widget/LinearLayout;

    invoke-virtual {v2, v1}, Landroid/widget/LinearLayout;->addView(Landroid/view/View;)V

    const-string v1, "\u0421\u0435\u0440\u0432\u0435\u0440\u044b"

    invoke-direct {p0, v1, v3}, Lorg/garlic/client/MainActivity;->button(Ljava/lang/String;Z)Landroid/widget/Button;

    move-result-object v1

    new-instance v2, Lorg/garlic/client/MainActivity$25;

    invoke-direct {v2, p0}, Lorg/garlic/client/MainActivity$25;-><init>(Lorg/garlic/client/MainActivity;)V

    invoke-virtual {v1, v2}, Landroid/widget/Button;->setOnClickListener(Landroid/view/View$OnClickListener;)V

    iget-object v2, p0, Lorg/garlic/client/MainActivity;->content:Landroid/widget/LinearLayout;

    invoke-virtual {v2, v1}, Landroid/widget/LinearLayout;->addView(Landroid/view/View;)V

    const-string v1, "\u041f\u043e\u043b\u044c\u0437\u043e\u0432\u0430\u0442\u0435\u043b\u0438 Legacy"

    invoke-direct {p0, v1, v3}, Lorg/garlic/client/MainActivity;->button(Ljava/lang/String;Z)Landroid/widget/Button;

    move-result-object v1

    new-instance v2, Lorg/garlic/client/MainActivity$26;

    invoke-direct {v2, p0}, Lorg/garlic/client/MainActivity$26;-><init>(Lorg/garlic/client/MainActivity;)V

    invoke-virtual {v1, v2}, Landroid/widget/Button;->setOnClickListener(Landroid/view/View$OnClickListener;)V

    iget-object v2, p0, Lorg/garlic/client/MainActivity;->content:Landroid/widget/LinearLayout;

    invoke-virtual {v2, v1}, Landroid/widget/LinearLayout;->addView(Landroid/view/View;)V

    .line 106
    :cond_5
    invoke-direct {p0, v0}, Lorg/garlic/client/MainActivity;->space(I)V

    const-string v0, "\u0412\u044b\u0439\u0442\u0438"

    invoke-direct {p0, v0, v3}, Lorg/garlic/client/MainActivity;->button(Ljava/lang/String;Z)Landroid/widget/Button;

    move-result-object v0

    new-instance v1, Lorg/garlic/client/MainActivity$27;

    invoke-direct {v1, p0}, Lorg/garlic/client/MainActivity$27;-><init>(Lorg/garlic/client/MainActivity;)V

    invoke-virtual {v0, v1}, Landroid/widget/Button;->setOnClickListener(Landroid/view/View$OnClickListener;)V

    iget-object v1, p0, Lorg/garlic/client/MainActivity;->content:Landroid/widget/LinearLayout;

    invoke-virtual {v1, v0}, Landroid/widget/LinearLayout;->addView(Landroid/view/View;)V

    .line 107
    invoke-static {p0}, Lorg/garlic/client/BackgroundSettings;->shouldStart(Landroid/app/Activity;)Z
    move-result v0
    if-eqz v0, :alpha_home_done
    sget-boolean v0, Lorg/garlic/client/GarlicProxyService;->isRunning:Z
    if-nez v0, :alpha_home_done
    sget-boolean v0, Lorg/garlic/client/GarlicProxyService;->isConnecting:Z
    if-nez v0, :alpha_home_done
    invoke-direct {p0}, Lorg/garlic/client/MainActivity;->startVpn()V
    :alpha_home_done
    return-void
.end method

.method private showLegacy()V
    .locals 7

    .line 127
    iget-boolean v0, p0, Lorg/garlic/client/MainActivity;->admin:Z

    if-nez v0, :cond_0

    return-void

    .line 128
    :cond_0
    iget v0, p0, Lorg/garlic/client/MainActivity;->generation:I

    .line 129
    new-instance v1, Lorg/garlic/client/LegacyUsers;

    iget-object v3, p0, Lorg/garlic/client/MainActivity;->worker:Ljava/util/concurrent/ExecutorService;

    invoke-direct {p0}, Lorg/garlic/client/MainActivity;->dir()Ljava/io/File;

    move-result-object v2

    invoke-virtual {v2}, Ljava/io/File;->getAbsolutePath()Ljava/lang/String;

    move-result-object v4

    invoke-direct {p0}, Lorg/garlic/client/MainActivity;->origins()Ljava/lang/String;

    move-result-object v5

    new-instance v6, Lorg/garlic/client/MainActivity$9;

    invoke-direct {v6, p0, v0}, Lorg/garlic/client/MainActivity$9;-><init>(Lorg/garlic/client/MainActivity;I)V

    move-object v2, p0

    invoke-direct/range {v1 .. v6}, Lorg/garlic/client/LegacyUsers;-><init>(Landroid/app/Activity;Ljava/util/concurrent/Executor;Ljava/lang/String;Ljava/lang/String;Ljava/util/function/BooleanSupplier;)V

    .line 130
    invoke-virtual {v1}, Lorg/garlic/client/LegacyUsers;->open()V

    .line 131
    return-void
.end method

.method private showLogin()V
    .locals 7

    .line 82
    const/4 v0, 0x0

    iput-boolean v0, p0, Lorg/garlic/client/MainActivity;->loggedIn:Z

    iput-boolean v0, p0, Lorg/garlic/client/MainActivity;->admin:Z

    const-string v1, ""

    invoke-direct {p0, v1}, Lorg/garlic/client/MainActivity;->shell(Ljava/lang/String;)V

    const/16 v2, 0x2a

    invoke-direct {p0, v2}, Lorg/garlic/client/MainActivity;->space(I)V

    const/16 v2, 0x1e

    iget v3, p0, Lorg/garlic/client/MainActivity;->ink:I

    const-string v4, "\u0412\u0430\u0448\u0430 \u0441\u0435\u0442\u044c.\n\u0412\u0430\u0448\u0435 \u043f\u043e\u0434\u043a\u043b\u044e\u0447\u0435\u043d\u0438\u0435."

    invoke-direct {p0, v4, v2, v3}, Lorg/garlic/client/MainActivity;->text(Ljava/lang/String;II)Landroid/widget/TextView;

    move-result-object v2

    sget-object v3, Landroid/graphics/Typeface;->DEFAULT:Landroid/graphics/Typeface;

    const/4 v4, 0x1

    invoke-virtual {v2, v3, v4}, Landroid/widget/TextView;->setTypeface(Landroid/graphics/Typeface;I)V

    iget-object v3, p0, Lorg/garlic/client/MainActivity;->content:Landroid/widget/LinearLayout;

    invoke-virtual {v3, v2}, Landroid/widget/LinearLayout;->addView(Landroid/view/View;)V

    iget-object v2, p0, Lorg/garlic/client/MainActivity;->content:Landroid/widget/LinearLayout;

    const/16 v3, 0x10

    iget v5, p0, Lorg/garlic/client/MainActivity;->muted:I

    const-string v6, "\u0423\u043a\u0430\u0436\u0438\u0442\u0435 \u043b\u044e\u0431\u043e\u0439 \u0441\u0435\u0440\u0432\u0435\u0440 Garlic \u0438 \u0441\u0432\u043e\u0439 \u043a\u043b\u044e\u0447."

    invoke-direct {p0, v6, v3, v5}, Lorg/garlic/client/MainActivity;->text(Ljava/lang/String;II)Landroid/widget/TextView;

    move-result-object v3

    invoke-virtual {v2, v3}, Landroid/widget/LinearLayout;->addView(Landroid/view/View;)V

    const/16 v2, 0x14

    invoke-direct {p0, v2}, Lorg/garlic/client/MainActivity;->space(I)V

    .line 83
    const-string v2, "\u0410\u0434\u0440\u0435\u0441 \u0441\u0435\u0440\u0432\u0435\u0440\u0430"

    invoke-direct {p0, v2, v0}, Lorg/garlic/client/MainActivity;->input(Ljava/lang/String;Z)Landroid/widget/EditText;

    move-result-object v2

    iput-object v2, p0, Lorg/garlic/client/MainActivity;->server:Landroid/widget/EditText;

    iget-object v2, p0, Lorg/garlic/client/MainActivity;->server:Landroid/widget/EditText;

    iget-object v3, p0, Lorg/garlic/client/MainActivity;->prefs:Landroid/content/SharedPreferences;

    const-string v5, "bootstrap"

    invoke-interface {v3, v5, v1}, Landroid/content/SharedPreferences;->getString(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;

    move-result-object v1

    invoke-virtual {v2, v1}, Landroid/widget/EditText;->setText(Ljava/lang/CharSequence;)V

    iget-object v1, p0, Lorg/garlic/client/MainActivity;->content:Landroid/widget/LinearLayout;

    iget-object v2, p0, Lorg/garlic/client/MainActivity;->server:Landroid/widget/EditText;

    invoke-virtual {v1, v2}, Landroid/widget/LinearLayout;->addView(Landroid/view/View;)V

    const-string v1, "\u041a\u043b\u044e\u0447 \u0434\u043e\u0441\u0442\u0443\u043f\u0430"

    invoke-direct {p0, v1, v4}, Lorg/garlic/client/MainActivity;->input(Ljava/lang/String;Z)Landroid/widget/EditText;

    move-result-object v1

    iput-object v1, p0, Lorg/garlic/client/MainActivity;->key:Landroid/widget/EditText;

    iget-object v1, p0, Lorg/garlic/client/MainActivity;->content:Landroid/widget/LinearLayout;

    iget-object v2, p0, Lorg/garlic/client/MainActivity;->key:Landroid/widget/EditText;

    invoke-virtual {v1, v2}, Landroid/widget/LinearLayout;->addView(Landroid/view/View;)V

    const/16 v1, 0xc

    invoke-direct {p0, v1}, Lorg/garlic/client/MainActivity;->space(I)V

    const-string v1, "\u041f\u0440\u043e\u0434\u043e\u043b\u0436\u0438\u0442\u044c"

    invoke-direct {p0, v1, v4}, Lorg/garlic/client/MainActivity;->button(Ljava/lang/String;Z)Landroid/widget/Button;

    move-result-object v1

    new-instance v2, Lorg/garlic/client/MainActivity$47;

    invoke-direct {v2, p0, v1}, Lorg/garlic/client/MainActivity$47;-><init>(Lorg/garlic/client/MainActivity;Landroid/widget/Button;)V

    invoke-virtual {v1, v2}, Landroid/widget/Button;->setOnClickListener(Landroid/view/View$OnClickListener;)V

    iget-object v2, p0, Lorg/garlic/client/MainActivity;->content:Landroid/widget/LinearLayout;

    invoke-virtual {v2, v1}, Landroid/widget/LinearLayout;->addView(Landroid/view/View;)V

    iget-object v2, p0, Lorg/garlic/client/MainActivity;->content:Landroid/widget/LinearLayout;

    iget-object v3, p0, Lorg/garlic/client/MainActivity;->message:Landroid/widget/TextView;

    invoke-virtual {v2, v3}, Landroid/widget/LinearLayout;->addView(Landroid/view/View;)V

    .line 84
    sget-object v2, Lorg/garlic/client/GarlicBridge;->loadError:Ljava/lang/String;

    if-eqz v2, :cond_0

    iget-object v2, p0, Lorg/garlic/client/MainActivity;->message:Landroid/widget/TextView;

    sget-object v3, Lorg/garlic/client/GarlicBridge;->loadError:Ljava/lang/String;

    new-instance v4, Ljava/lang/StringBuilder;

    invoke-direct {v4}, Ljava/lang/StringBuilder;-><init>()V

    const-string v5, "\u041d\u0435 \u0443\u0434\u0430\u043b\u043e\u0441\u044c \u0437\u0430\u0433\u0440\u0443\u0437\u0438\u0442\u044c \u0441\u0435\u0442\u0435\u0432\u043e\u0439 \u043c\u043e\u0434\u0443\u043b\u044c: "

    invoke-virtual {v4, v5}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v4

    invoke-virtual {v4, v3}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v3

    invoke-virtual {v3}, Ljava/lang/StringBuilder;->toString()Ljava/lang/String;

    move-result-object v3

    invoke-virtual {v2, v3}, Landroid/widget/TextView;->setText(Ljava/lang/CharSequence;)V

    invoke-virtual {v1, v0}, Landroid/widget/Button;->setEnabled(Z)V

    .line 85
    :cond_0
    iget-object v0, p0, Lorg/garlic/client/MainActivity;->content:Landroid/widget/LinearLayout;
    invoke-static {p0, v0}, Lorg/garlic/client/BackgroundSettings;->addExportButton(Landroid/app/Activity;Landroid/widget/LinearLayout;)V
    return-void
.end method

.method private showServers()V
    .locals 4

    .line 133
    iget v0, p0, Lorg/garlic/client/MainActivity;->generation:I

    .line 134
    iget-boolean v1, p0, Lorg/garlic/client/MainActivity;->admin:Z

    if-nez v1, :cond_0

    return-void

    :cond_0
    const-string v1, "\u0421\u0435\u0440\u0432\u0435\u0440\u044b \u0441\u0435\u0442\u0438"

    invoke-direct {p0, v1}, Lorg/garlic/client/MainActivity;->shell(Ljava/lang/String;)V

    const-string v1, "\u041d\u0430\u0437\u0430\u0434"

    const/4 v2, 0x0

    invoke-direct {p0, v1, v2}, Lorg/garlic/client/MainActivity;->button(Ljava/lang/String;Z)Landroid/widget/Button;

    move-result-object v1

    new-instance v3, Lorg/garlic/client/MainActivity$6;

    invoke-direct {v3, p0}, Lorg/garlic/client/MainActivity$6;-><init>(Lorg/garlic/client/MainActivity;)V

    invoke-virtual {v1, v3}, Landroid/widget/Button;->setOnClickListener(Landroid/view/View$OnClickListener;)V

    iget-object v3, p0, Lorg/garlic/client/MainActivity;->content:Landroid/widget/LinearLayout;

    invoke-virtual {v3, v1}, Landroid/widget/LinearLayout;->addView(Landroid/view/View;)V

    .line 135
    const-string v1, "\u041e\u0431\u043d\u043e\u0432\u0438\u0442\u044c"

    invoke-direct {p0, v1, v2}, Lorg/garlic/client/MainActivity;->button(Ljava/lang/String;Z)Landroid/widget/Button;

    move-result-object v1

    new-instance v2, Lorg/garlic/client/MainActivity$7;

    invoke-direct {v2, p0}, Lorg/garlic/client/MainActivity$7;-><init>(Lorg/garlic/client/MainActivity;)V

    invoke-virtual {v1, v2}, Landroid/widget/Button;->setOnClickListener(Landroid/view/View$OnClickListener;)V

    iget-object v2, p0, Lorg/garlic/client/MainActivity;->content:Landroid/widget/LinearLayout;

    invoke-virtual {v2, v1}, Landroid/widget/LinearLayout;->addView(Landroid/view/View;)V

    iget-object v1, p0, Lorg/garlic/client/MainActivity;->content:Landroid/widget/LinearLayout;

    iget-object v2, p0, Lorg/garlic/client/MainActivity;->message:Landroid/widget/TextView;

    invoke-virtual {v1, v2}, Landroid/widget/LinearLayout;->addView(Landroid/view/View;)V

    iget-object v1, p0, Lorg/garlic/client/MainActivity;->message:Landroid/widget/TextView;

    const-string v2, "\u041f\u043e\u043b\u0443\u0447\u0430\u0435\u043c \u0441\u0432\u0435\u0434\u0435\u043d\u0438\u044f \u043e\u0442 \u0441\u0435\u0442\u0438\u2026"

    invoke-virtual {v1, v2}, Landroid/widget/TextView;->setText(Ljava/lang/CharSequence;)V

    .line 136
    new-instance v1, Landroid/widget/LinearLayout;

    invoke-direct {v1, p0}, Landroid/widget/LinearLayout;-><init>(Landroid/content/Context;)V

    const/4 v2, 0x1

    invoke-virtual {v1, v2}, Landroid/widget/LinearLayout;->setOrientation(I)V

    iget-object v2, p0, Lorg/garlic/client/MainActivity;->content:Landroid/widget/LinearLayout;

    invoke-virtual {v2, v1}, Landroid/widget/LinearLayout;->addView(Landroid/view/View;)V

    .line 137
    iget-object v2, p0, Lorg/garlic/client/MainActivity;->worker:Ljava/util/concurrent/ExecutorService;

    new-instance v3, Lorg/garlic/client/MainActivity$8;

    invoke-direct {v3, p0, v0, v1}, Lorg/garlic/client/MainActivity$8;-><init>(Lorg/garlic/client/MainActivity;ILandroid/widget/LinearLayout;)V

    invoke-interface {v2, v3}, Ljava/util/concurrent/ExecutorService;->execute(Ljava/lang/Runnable;)V

    .line 149
    return-void
.end method

.method private showUsers()V
    .locals 4

    .line 157
    iget v0, p0, Lorg/garlic/client/MainActivity;->generation:I

    iget-boolean v1, p0, Lorg/garlic/client/MainActivity;->admin:Z

    if-nez v1, :cond_0

    return-void

    :cond_0
    const-string v1, "\u0423\u043f\u0440\u0430\u0432\u043b\u0435\u043d\u0438\u0435 \u043f\u043e\u043b\u044c\u0437\u043e\u0432\u0430\u0442\u0435\u043b\u044f\u043c\u0438"

    invoke-direct {p0, v1}, Lorg/garlic/client/MainActivity;->shell(Ljava/lang/String;)V

    const-string v1, "\u041d\u0430\u0437\u0430\u0434"

    const/4 v2, 0x0

    invoke-direct {p0, v1, v2}, Lorg/garlic/client/MainActivity;->button(Ljava/lang/String;Z)Landroid/widget/Button;

    move-result-object v1

    new-instance v2, Lorg/garlic/client/MainActivity$3;

    invoke-direct {v2, p0}, Lorg/garlic/client/MainActivity$3;-><init>(Lorg/garlic/client/MainActivity;)V

    invoke-virtual {v1, v2}, Landroid/widget/Button;->setOnClickListener(Landroid/view/View$OnClickListener;)V

    iget-object v2, p0, Lorg/garlic/client/MainActivity;->content:Landroid/widget/LinearLayout;

    invoke-virtual {v2, v1}, Landroid/widget/LinearLayout;->addView(Landroid/view/View;)V

    const-string v1, "\u0414\u043e\u0431\u0430\u0432\u0438\u0442\u044c \u043f\u043e\u043b\u044c\u0437\u043e\u0432\u0430\u0442\u0435\u043b\u044f"

    const/4 v2, 0x1

    invoke-direct {p0, v1, v2}, Lorg/garlic/client/MainActivity;->button(Ljava/lang/String;Z)Landroid/widget/Button;

    move-result-object v1

    new-instance v3, Lorg/garlic/client/MainActivity$4;

    invoke-direct {v3, p0}, Lorg/garlic/client/MainActivity$4;-><init>(Lorg/garlic/client/MainActivity;)V

    invoke-virtual {v1, v3}, Landroid/widget/Button;->setOnClickListener(Landroid/view/View$OnClickListener;)V

    iget-object v3, p0, Lorg/garlic/client/MainActivity;->content:Landroid/widget/LinearLayout;

    invoke-virtual {v3, v1}, Landroid/widget/LinearLayout;->addView(Landroid/view/View;)V

    iget-object v1, p0, Lorg/garlic/client/MainActivity;->content:Landroid/widget/LinearLayout;

    iget-object v3, p0, Lorg/garlic/client/MainActivity;->message:Landroid/widget/TextView;

    invoke-virtual {v1, v3}, Landroid/widget/LinearLayout;->addView(Landroid/view/View;)V

    new-instance v1, Landroid/widget/LinearLayout;

    invoke-direct {v1, p0}, Landroid/widget/LinearLayout;-><init>(Landroid/content/Context;)V

    invoke-virtual {v1, v2}, Landroid/widget/LinearLayout;->setOrientation(I)V

    iget-object v2, p0, Lorg/garlic/client/MainActivity;->content:Landroid/widget/LinearLayout;

    invoke-virtual {v2, v1}, Landroid/widget/LinearLayout;->addView(Landroid/view/View;)V

    iget-object v2, p0, Lorg/garlic/client/MainActivity;->message:Landroid/widget/TextView;

    const-string v3, "\u0417\u0430\u0433\u0440\u0443\u0437\u043a\u0430\u2026"

    invoke-virtual {v2, v3}, Landroid/widget/TextView;->setText(Ljava/lang/CharSequence;)V

    iget-object v2, p0, Lorg/garlic/client/MainActivity;->worker:Ljava/util/concurrent/ExecutorService;

    new-instance v3, Lorg/garlic/client/MainActivity$5;

    invoke-direct {v3, p0, v0, v1}, Lorg/garlic/client/MainActivity$5;-><init>(Lorg/garlic/client/MainActivity;ILandroid/widget/LinearLayout;)V

    invoke-interface {v2, v3}, Ljava/util/concurrent/ExecutorService;->execute(Ljava/lang/Runnable;)V

    return-void
.end method

.method private space(I)V
    .locals 4

    .line 57
    new-instance v0, Landroid/view/View;

    invoke-direct {v0, p0}, Landroid/view/View;-><init>(Landroid/content/Context;)V

    iget-object v1, p0, Lorg/garlic/client/MainActivity;->content:Landroid/widget/LinearLayout;

    new-instance v2, Landroid/widget/LinearLayout$LayoutParams;

    const/4 v3, 0x1

    invoke-direct {p0, p1}, Lorg/garlic/client/MainActivity;->dp(I)I

    move-result p1

    invoke-direct {v2, v3, p1}, Landroid/widget/LinearLayout$LayoutParams;-><init>(II)V

    invoke-virtual {v1, v0, v2}, Landroid/widget/LinearLayout;->addView(Landroid/view/View;Landroid/view/ViewGroup$LayoutParams;)V

    return-void
.end method

.method private startVpn()V
    .locals 3

    .line 109
    iget-boolean v0, p0, Lorg/garlic/client/MainActivity;->loggedIn:Z

    if-nez v0, :cond_0

    return-void

    :cond_0
    iget v0, p0, Lorg/garlic/client/MainActivity;->generation:I

    iput v0, p0, Lorg/garlic/client/MainActivity;->vpnPermissionGeneration:I

    const/4 v0, 0x0

    :try_start_0
    sput-object v0, Lorg/garlic/client/GarlicProxyService;->lastError:Ljava/lang/String;

    invoke-static {p0}, Landroid/net/VpnService;->prepare(Landroid/content/Context;)Landroid/content/Intent;

    move-result-object v1

    const/16 v2, 0x64

    if-eqz v1, :cond_1

    invoke-virtual {p0, v1, v2}, Lorg/garlic/client/MainActivity;->startActivityForResult(Landroid/content/Intent;I)V

    goto :goto_0

    :cond_1
    const/4 v1, -0x1

    invoke-virtual {p0, v2, v1, v0}, Lorg/garlic/client/MainActivity;->onActivityResult(IILandroid/content/Intent;)V
    :try_end_0
    .catch Ljava/lang/Exception; {:try_start_0 .. :try_end_0} :catch_0

    goto :goto_0

    :catch_0
    move-exception v0

    iget-object v0, p0, Lorg/garlic/client/MainActivity;->message:Landroid/widget/TextView;

    const-string v1, "\u041d\u0435 \u0443\u0434\u0430\u043b\u043e\u0441\u044c \u043f\u043e\u043b\u0443\u0447\u0438\u0442\u044c \u0440\u0430\u0437\u0440\u0435\u0448\u0435\u043d\u0438\u0435 VPN"

    invoke-virtual {v0, v1}, Landroid/widget/TextView;->setText(Ljava/lang/CharSequence;)V

    :goto_0
    return-void
.end method

.method private stopVpn()V
    .locals 2

    .line 112
    const/4 v0, -0x1

    iput v0, p0, Lorg/garlic/client/MainActivity;->vpnPermissionGeneration:I

    const/4 v0, 0x0

    iput-boolean v0, p0, Lorg/garlic/client/MainActivity;->starting:Z

    .line 113
    new-instance v0, Landroid/content/Intent;

    const-class v1, Lorg/garlic/client/GarlicProxyService;

    invoke-direct {v0, p0, v1}, Landroid/content/Intent;-><init>(Landroid/content/Context;Ljava/lang/Class;)V

    const-string v1, "STOP"

    invoke-virtual {v0, v1}, Landroid/content/Intent;->setAction(Ljava/lang/String;)Landroid/content/Intent;

    .line 114
    :try_start_0
    invoke-virtual {p0, v0}, Lorg/garlic/client/MainActivity;->startService(Landroid/content/Intent;)Landroid/content/ComponentName;
    :try_end_0
    .catch Ljava/lang/Exception; {:try_start_0 .. :try_end_0} :catch_0

    goto :goto_0

    :catch_0
    move-exception v1

    invoke-virtual {p0, v0}, Lorg/garlic/client/MainActivity;->stopService(Landroid/content/Intent;)Z

    .line 115
    :goto_0
    return-void
.end method

.method private subscription()Ljava/lang/String;
    .locals 9

    .line 98
    iget-boolean v0, p0, Lorg/garlic/client/MainActivity;->admin:Z

    if-eqz v0, :cond_0

    const-string v0, "\u0410\u0434\u043c\u0438\u043d\u0438\u0441\u0442\u0440\u0430\u0442\u043e\u0440 \u00b7 \u0411\u0435\u0437\u043b\u0438\u043c\u0438\u0442"

    return-object v0

    :cond_0
    iget-object v0, p0, Lorg/garlic/client/MainActivity;->account:Lorg/json/JSONObject;

    const-string v1, "\u041f\u043e\u0434\u043f\u0438\u0441\u043a\u0430 \u043d\u0435 \u043f\u0440\u043e\u0432\u0435\u0440\u0435\u043d\u0430"

    if-nez v0, :cond_1

    return-object v1

    :cond_1
    iget-object v0, p0, Lorg/garlic/client/MainActivity;->account:Lorg/json/JSONObject;

    const-string v2, "subscription"

    invoke-virtual {v0, v2}, Lorg/json/JSONObject;->optJSONObject(Ljava/lang/String;)Lorg/json/JSONObject;

    move-result-object v0

    if-nez v0, :cond_2

    return-object v1

    :cond_2
    const-string v1, "kind"

    invoke-virtual {v0, v1}, Lorg/json/JSONObject;->optString(Ljava/lang/String;)Ljava/lang/String;

    move-result-object v1

    const-string v2, "unlimited"

    invoke-virtual {v2, v1}, Ljava/lang/String;->equals(Ljava/lang/Object;)Z

    move-result v1

    if-eqz v1, :cond_3

    const-string v0, "\u041f\u043e\u0434\u043f\u0438\u0441\u043a\u0430 \u00b7 \u0411\u0435\u0437\u043b\u0438\u043c\u0438\u0442"

    return-object v0

    :cond_3
    const-string v1, "expires_at"

    invoke-virtual {v0, v1}, Lorg/json/JSONObject;->optLong(Ljava/lang/String;)J

    move-result-wide v2

    invoke-static {}, Ljava/lang/System;->currentTimeMillis()J

    move-result-wide v4

    const-wide/16 v6, 0x3e8

    div-long/2addr v4, v6

    sub-long/2addr v2, v4

    const-wide/16 v4, 0x0

    cmp-long v4, v2, v4

    if-gtz v4, :cond_4

    const-string v0, "\u041f\u043e\u0434\u043f\u0438\u0441\u043a\u0430 \u0438\u0441\u0442\u0435\u043a\u043b\u0430"

    return-object v0

    :cond_4
    new-instance v4, Ljava/util/Locale;

    const-string v5, "ru"

    const-string v8, "RU"

    invoke-direct {v4, v5, v8}, Ljava/util/Locale;-><init>(Ljava/lang/String;Ljava/lang/String;)V

    const/4 v5, 0x1

    invoke-static {v5, v4}, Ljava/text/DateFormat;->getDateInstance(ILjava/util/Locale;)Ljava/text/DateFormat;

    move-result-object v4

    new-instance v5, Ljava/util/Date;

    invoke-virtual {v0, v1}, Lorg/json/JSONObject;->optLong(Ljava/lang/String;)J

    move-result-wide v0

    mul-long/2addr v0, v6

    invoke-direct {v5, v0, v1}, Ljava/util/Date;-><init>(J)V

    invoke-virtual {v4, v5}, Ljava/text/DateFormat;->format(Ljava/util/Date;)Ljava/lang/String;

    move-result-object v0

    const-wide/32 v4, 0x1517f

    add-long/2addr v2, v4

    const-wide/32 v4, 0x15180

    div-long/2addr v2, v4

    new-instance v1, Ljava/lang/StringBuilder;

    invoke-direct {v1}, Ljava/lang/StringBuilder;-><init>()V

    const-string v4, "\u0410\u043a\u0442\u0438\u0432\u043d\u0430 \u0434\u043e "

    invoke-virtual {v1, v4}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v1

    invoke-virtual {v1, v0}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v0

    const-string v1, " \u00b7 \u043e\u0441\u0442\u0430\u043b\u043e\u0441\u044c \u0434\u043d\u0435\u0439: "

    invoke-virtual {v0, v1}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v0

    invoke-virtual {v0, v2, v3}, Ljava/lang/StringBuilder;->append(J)Ljava/lang/StringBuilder;

    move-result-object v0

    invoke-virtual {v0}, Ljava/lang/StringBuilder;->toString()Ljava/lang/String;

    move-result-object v0

    return-object v0
.end method

.method private subscriptionDialog(Lorg/json/JSONObject;)V
    .locals 12

    .line 159
    const/4 v0, 0x0

    if-nez p1, :cond_0

    move-object v5, v0

    goto :goto_0

    :cond_0
    const-string v1, "username"

    invoke-virtual {p1, v1}, Lorg/json/JSONObject;->optString(Ljava/lang/String;)Ljava/lang/String;

    move-result-object v1

    move-object v5, v1

    .line 160
    :goto_0
    new-instance v1, Landroid/widget/LinearLayout;

    invoke-direct {v1, p0}, Landroid/widget/LinearLayout;-><init>(Landroid/content/Context;)V

    const/4 v2, 0x1

    invoke-virtual {v1, v2}, Landroid/widget/LinearLayout;->setOrientation(I)V

    const/16 v3, 0x18

    invoke-direct {p0, v3}, Lorg/garlic/client/MainActivity;->dp(I)I

    move-result v4

    invoke-direct {p0, v3}, Lorg/garlic/client/MainActivity;->dp(I)I

    move-result v3

    const/4 v6, 0x0

    invoke-virtual {v1, v4, v6, v3, v6}, Landroid/widget/LinearLayout;->setPadding(IIII)V

    .line 161
    if-nez v5, :cond_1

    const-string v3, "\u0418\u043c\u044f \u043f\u043e\u043b\u044c\u0437\u043e\u0432\u0430\u0442\u0435\u043b\u044f"

    invoke-direct {p0, v3, v6}, Lorg/garlic/client/MainActivity;->input(Ljava/lang/String;Z)Landroid/widget/EditText;

    move-result-object v3

    goto :goto_1

    :cond_1
    move-object v3, v0

    :goto_1
    if-eqz v3, :cond_2

    invoke-virtual {v1, v3}, Landroid/widget/LinearLayout;->addView(Landroid/view/View;)V

    .line 162
    :cond_2
    invoke-static {}, Ljava/util/Calendar;->getInstance()Ljava/util/Calendar;

    move-result-object v4

    const/4 v7, 0x2

    invoke-virtual {v4, v7, v2}, Ljava/util/Calendar;->add(II)V

    .line 163
    if-nez p1, :cond_3

    move-object p1, v0

    goto :goto_2

    :cond_3
    const-string v7, "subscription"

    invoke-virtual {p1, v7}, Lorg/json/JSONObject;->optJSONObject(Ljava/lang/String;)Lorg/json/JSONObject;

    move-result-object p1

    :goto_2
    if-eqz p1, :cond_4

    const-string v7, "expires_at"

    invoke-virtual {p1, v7}, Lorg/json/JSONObject;->optLong(Ljava/lang/String;)J

    move-result-wide v8

    const-wide/16 v10, 0x0

    cmp-long v8, v8, v10

    if-lez v8, :cond_4

    invoke-virtual {p1, v7}, Lorg/json/JSONObject;->optLong(Ljava/lang/String;)J

    move-result-wide v7

    const-wide/16 v9, 0x3e8

    mul-long/2addr v7, v9

    invoke-virtual {v4, v7, v8}, Ljava/util/Calendar;->setTimeInMillis(J)V

    .line 164
    :cond_4
    new-instance v7, Ljava/util/Locale;

    const-string v8, "ru"

    const-string v9, "RU"

    invoke-direct {v7, v8, v9}, Ljava/util/Locale;-><init>(Ljava/lang/String;Ljava/lang/String;)V

    invoke-static {v2, v7}, Ljava/text/DateFormat;->getDateInstance(ILjava/util/Locale;)Ljava/text/DateFormat;

    move-result-object v7

    invoke-virtual {v4}, Ljava/util/Calendar;->getTime()Ljava/util/Date;

    move-result-object v8

    invoke-virtual {v7, v8}, Ljava/text/DateFormat;->format(Ljava/util/Date;)Ljava/lang/String;

    move-result-object v7

    new-instance v8, Ljava/lang/StringBuilder;

    invoke-direct {v8}, Ljava/lang/StringBuilder;-><init>()V

    const-string v9, "\u0410\u043a\u0442\u0438\u0432\u043d\u0430 \u0434\u043e "

    invoke-virtual {v8, v9}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v8

    invoke-virtual {v8, v7}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v7

    invoke-virtual {v7}, Ljava/lang/StringBuilder;->toString()Ljava/lang/String;

    move-result-object v7

    invoke-direct {p0, v7, v6}, Lorg/garlic/client/MainActivity;->button(Ljava/lang/String;Z)Landroid/widget/Button;

    move-result-object v7

    .line 165
    new-instance v8, Lorg/garlic/client/MainActivity$31;

    invoke-direct {v8, p0, v4, v7}, Lorg/garlic/client/MainActivity$31;-><init>(Lorg/garlic/client/MainActivity;Ljava/util/Calendar;Landroid/widget/Button;)V

    invoke-virtual {v7, v8}, Landroid/widget/Button;->setOnClickListener(Landroid/view/View$OnClickListener;)V

    invoke-virtual {v1, v7}, Landroid/widget/LinearLayout;->addView(Landroid/view/View;)V

    .line 166
    move-object v8, v7

    new-instance v7, Landroid/widget/CheckBox;

    invoke-direct {v7, p0}, Landroid/widget/CheckBox;-><init>(Landroid/content/Context;)V

    const-string v9, "\u0411\u0435\u0437 \u043e\u0433\u0440\u0430\u043d\u0438\u0447\u0435\u043d\u0438\u044f \u0441\u0440\u043e\u043a\u0430"

    invoke-virtual {v7, v9}, Landroid/widget/CheckBox;->setText(Ljava/lang/CharSequence;)V

    iget v9, p0, Lorg/garlic/client/MainActivity;->ink:I

    invoke-virtual {v7, v9}, Landroid/widget/CheckBox;->setTextColor(I)V

    new-instance v9, Lorg/garlic/client/MainActivity$32;

    invoke-direct {v9, v8}, Lorg/garlic/client/MainActivity$32;-><init>(Landroid/widget/Button;)V

    invoke-virtual {v7, v9}, Landroid/widget/CheckBox;->setOnCheckedChangeListener(Landroid/widget/CompoundButton$OnCheckedChangeListener;)V

    if-eqz p1, :cond_5

    const-string v8, "kind"

    invoke-virtual {p1, v8}, Lorg/json/JSONObject;->optString(Ljava/lang/String;)Ljava/lang/String;

    move-result-object p1

    const-string v8, "unlimited"

    invoke-virtual {v8, p1}, Ljava/lang/String;->equals(Ljava/lang/Object;)Z

    move-result p1

    if-eqz p1, :cond_5

    goto :goto_3

    :cond_5
    move v2, v6

    :goto_3
    invoke-virtual {v7, v2}, Landroid/widget/CheckBox;->setChecked(Z)V

    invoke-virtual {v1, v7}, Landroid/widget/LinearLayout;->addView(Landroid/view/View;)V

    const/16 p1, 0xd

    iget v2, p0, Lorg/garlic/client/MainActivity;->muted:I

    const-string v6, "\u0412\u044b\u0431\u0440\u0430\u043d\u043d\u0430\u044f \u0434\u0430\u0442\u0430 \u0432\u043a\u043b\u044e\u0447\u0438\u0442\u0435\u043b\u044c\u043d\u043e, \u043f\u043e \u0432\u0440\u0435\u043c\u0435\u043d\u0438 \u0443\u0441\u0442\u0440\u043e\u0439\u0441\u0442\u0432\u0430"

    invoke-direct {p0, v6, p1, v2}, Lorg/garlic/client/MainActivity;->text(Ljava/lang/String;II)Landroid/widget/TextView;

    move-result-object p1

    invoke-virtual {v1, p1}, Landroid/widget/LinearLayout;->addView(Landroid/view/View;)V

    .line 167
    new-instance p1, Landroid/app/AlertDialog$Builder;

    invoke-direct {p1, p0}, Landroid/app/AlertDialog$Builder;-><init>(Landroid/content/Context;)V

    if-nez v5, :cond_6

    const-string v2, "\u041d\u043e\u0432\u044b\u0439 \u043f\u043e\u043b\u044c\u0437\u043e\u0432\u0430\u0442\u0435\u043b\u044c"

    goto :goto_4

    :cond_6
    new-instance v2, Ljava/lang/StringBuilder;

    invoke-direct {v2}, Ljava/lang/StringBuilder;-><init>()V

    const-string v6, "\u041f\u043e\u0434\u043f\u0438\u0441\u043a\u0430 \u00b7 "

    invoke-virtual {v2, v6}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v2

    invoke-virtual {v2, v5}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v2

    invoke-virtual {v2}, Ljava/lang/StringBuilder;->toString()Ljava/lang/String;

    move-result-object v2

    :goto_4
    invoke-virtual {p1, v2}, Landroid/app/AlertDialog$Builder;->setTitle(Ljava/lang/CharSequence;)Landroid/app/AlertDialog$Builder;

    move-result-object p1

    invoke-virtual {p1, v1}, Landroid/app/AlertDialog$Builder;->setView(Landroid/view/View;)Landroid/app/AlertDialog$Builder;

    move-result-object p1

    const-string v1, "\u041e\u0442\u043c\u0435\u043d\u0430"

    invoke-virtual {p1, v1, v0}, Landroid/app/AlertDialog$Builder;->setNegativeButton(Ljava/lang/CharSequence;Landroid/content/DialogInterface$OnClickListener;)Landroid/app/AlertDialog$Builder;

    move-result-object p1

    new-instance v2, Lorg/garlic/client/MainActivity$33;

    move-object v6, v3

    move-object v3, p0

    invoke-direct/range {v2 .. v7}, Lorg/garlic/client/MainActivity$33;-><init>(Lorg/garlic/client/MainActivity;Ljava/util/Calendar;Ljava/lang/String;Landroid/widget/EditText;Landroid/widget/CheckBox;)V

    const-string v0, "\u0421\u043e\u0445\u0440\u0430\u043d\u0438\u0442\u044c"

    invoke-virtual {p1, v0, v2}, Landroid/app/AlertDialog$Builder;->setPositiveButton(Ljava/lang/CharSequence;Landroid/content/DialogInterface$OnClickListener;)Landroid/app/AlertDialog$Builder;

    move-result-object p1

    .line 169
    invoke-virtual {p1}, Landroid/app/AlertDialog$Builder;->show()Landroid/app/AlertDialog;

    .line 170
    return-void
.end method

.method private takeover()V
    .locals 3

    .line 116
    iget-boolean v0, p0, Lorg/garlic/client/MainActivity;->loggedIn:Z

    if-nez v0, :cond_0

    return-void

    :cond_0
    sget-boolean v0, Lorg/garlic/client/GarlicProxyService;->isRunning:Z

    if-nez v0, :cond_2

    sget-boolean v0, Lorg/garlic/client/GarlicProxyService;->isConnecting:Z

    if-eqz v0, :cond_1

    goto :goto_0

    :cond_1
    iget v0, p0, Lorg/garlic/client/MainActivity;->generation:I

    iget-object v1, p0, Lorg/garlic/client/MainActivity;->message:Landroid/widget/TextView;

    const-string v2, "\u041f\u0435\u0440\u0435\u043a\u043b\u044e\u0447\u0430\u0435\u043c \u0434\u043e\u0441\u0442\u0443\u043f\u2026"

    invoke-virtual {v1, v2}, Landroid/widget/TextView;->setText(Ljava/lang/CharSequence;)V

    iget-object v1, p0, Lorg/garlic/client/MainActivity;->worker:Ljava/util/concurrent/ExecutorService;

    new-instance v2, Lorg/garlic/client/MainActivity$38;

    invoke-direct {v2, p0, v0}, Lorg/garlic/client/MainActivity$38;-><init>(Lorg/garlic/client/MainActivity;I)V

    invoke-interface {v1, v2}, Ljava/util/concurrent/ExecutorService;->execute(Ljava/lang/Runnable;)V

    return-void

    :cond_2
    :goto_0
    iget-object v0, p0, Lorg/garlic/client/MainActivity;->message:Landroid/widget/TextView;

    const-string v1, "\u042d\u0442\u043e \u0443\u0441\u0442\u0440\u043e\u0439\u0441\u0442\u0432\u043e \u0443\u0436\u0435 \u043f\u043e\u0434\u043a\u043b\u044e\u0447\u0435\u043d\u043e"

    invoke-virtual {v0, v1}, Landroid/widget/TextView;->setText(Ljava/lang/CharSequence;)V

    return-void
.end method

.method private text(Ljava/lang/String;II)Landroid/widget/TextView;
    .locals 1

    .line 56
    new-instance v0, Landroid/widget/TextView;

    invoke-direct {v0, p0}, Landroid/widget/TextView;-><init>(Landroid/content/Context;)V

    invoke-virtual {v0, p1}, Landroid/widget/TextView;->setText(Ljava/lang/CharSequence;)V

    int-to-float p1, p2

    invoke-virtual {v0, p1}, Landroid/widget/TextView;->setTextSize(F)V

    invoke-virtual {v0, p3}, Landroid/widget/TextView;->setTextColor(I)V

    const/16 p1, 0x8

    invoke-direct {p0, p1}, Lorg/garlic/client/MainActivity;->dp(I)I

    move-result p2

    invoke-direct {p0, p1}, Lorg/garlic/client/MainActivity;->dp(I)I

    move-result p1

    const/4 p3, 0x0

    invoke-virtual {v0, p3, p2, p3, p1}, Landroid/widget/TextView;->setPadding(IIII)V

    return-object v0
.end method

.method private updateText(Landroid/widget/TextView;Ljava/lang/String;)V
    .locals 1

    .line 53
    if-eqz p1, :cond_0

    invoke-virtual {p1}, Landroid/widget/TextView;->getText()Ljava/lang/CharSequence;

    move-result-object v0

    invoke-virtual {p2, v0}, Ljava/lang/String;->contentEquals(Ljava/lang/CharSequence;)Z

    move-result v0

    if-nez v0, :cond_0

    invoke-virtual {p1, p2}, Landroid/widget/TextView;->setText(Ljava/lang/CharSequence;)V

    :cond_0
    return-void
.end method

.method private urls(Lorg/json/JSONArray;)Ljava/lang/String;
    .locals 3
    .annotation system Ldalvik/annotation/Throws;
        value = {
            Ljava/lang/Exception;
        }
    .end annotation

    .line 62
    new-instance v0, Ljava/lang/StringBuilder;

    invoke-direct {v0}, Ljava/lang/StringBuilder;-><init>()V

    const/4 v1, 0x0

    :goto_0
    invoke-virtual {p1}, Lorg/json/JSONArray;->length()I

    move-result v2

    if-ge v1, v2, :cond_1

    if-lez v1, :cond_0

    const/16 v2, 0x2c

    invoke-virtual {v0, v2}, Ljava/lang/StringBuilder;->append(C)Ljava/lang/StringBuilder;

    :cond_0
    invoke-virtual {p1, v1}, Lorg/json/JSONArray;->getString(I)Ljava/lang/String;

    move-result-object v2

    invoke-virtual {v0, v2}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    add-int/lit8 v1, v1, 0x1

    goto :goto_0

    :cond_1
    invoke-virtual {v0}, Ljava/lang/StringBuilder;->toString()Ljava/lang/String;

    move-result-object p1

    return-object p1
.end method

.method private userActions(Lorg/json/JSONObject;)V
    .locals 8

    .line 173
    const-string v0, "username"

    invoke-virtual {p1, v0}, Lorg/json/JSONObject;->optString(Ljava/lang/String;)Ljava/lang/String;

    move-result-object v0

    const-string v1, "enabled"

    invoke-virtual {p1, v1}, Lorg/json/JSONObject;->optBoolean(Ljava/lang/String;)Z

    move-result v1

    .line 174
    new-instance v2, Landroid/app/AlertDialog$Builder;

    invoke-direct {v2, p0}, Landroid/app/AlertDialog$Builder;-><init>(Landroid/content/Context;)V

    invoke-virtual {v2, v0}, Landroid/app/AlertDialog$Builder;->setTitle(Ljava/lang/CharSequence;)Landroid/app/AlertDialog$Builder;

    move-result-object v2

    if-eqz v1, :cond_0

    const-string v3, "\u041e\u0442\u043a\u043b\u044e\u0447\u0438\u0442\u044c"

    goto :goto_0

    :cond_0
    const-string v3, "\u0412\u043a\u043b\u044e\u0447\u0438\u0442\u044c"

    :goto_0
    const-string v4, "\u0418\u0437\u043c\u0435\u043d\u0438\u0442\u044c \u0440\u043e\u043b\u044c"

    const-string v5, "\u0423\u0434\u0430\u043b\u0438\u0442\u044c"

    const-string v6, "\u0412\u044b\u043f\u0443\u0441\u0442\u0438\u0442\u044c \u043a\u043b\u044e\u0447"

    const-string v7, "\u0418\u0437\u043c\u0435\u043d\u0438\u0442\u044c \u043f\u043e\u0434\u043f\u0438\u0441\u043a\u0443"

    filled-new-array {v6, v7, v3, v4, v5}, [Ljava/lang/String;

    move-result-object v3

    new-instance v4, Lorg/garlic/client/MainActivity$12;

    invoke-direct {v4, p0, p1, v0, v1}, Lorg/garlic/client/MainActivity$12;-><init>(Lorg/garlic/client/MainActivity;Lorg/json/JSONObject;Ljava/lang/String;Z)V

    invoke-virtual {v2, v3, v4}, Landroid/app/AlertDialog$Builder;->setItems([Ljava/lang/CharSequence;Landroid/content/DialogInterface$OnClickListener;)Landroid/app/AlertDialog$Builder;

    move-result-object p1

    .line 179
    invoke-virtual {p1}, Landroid/app/AlertDialog$Builder;->show()Landroid/app/AlertDialog;

    .line 180
    return-void
.end method

.method private userSubscription(Lorg/json/JSONObject;)Ljava/lang/String;
    .locals 6

    .line 92
    const-string v0, "role"

    invoke-virtual {p1, v0}, Lorg/json/JSONObject;->optString(Ljava/lang/String;)Ljava/lang/String;

    move-result-object v0

    const-string v1, "admin"

    invoke-virtual {v1, v0}, Ljava/lang/String;->equals(Ljava/lang/Object;)Z

    move-result v0

    if-eqz v0, :cond_0

    const-string p1, "\u0410\u0434\u043c\u0438\u043d\u0438\u0441\u0442\u0440\u0430\u0442\u043e\u0440 \u00b7 \u0411\u0435\u0437\u043b\u0438\u043c\u0438\u0442"

    return-object p1

    .line 93
    :cond_0
    const-string v0, "subscription"

    invoke-virtual {p1, v0}, Lorg/json/JSONObject;->optJSONObject(Ljava/lang/String;)Lorg/json/JSONObject;

    move-result-object p1

    .line 94
    if-nez p1, :cond_1

    const-string p1, "\u0421\u0440\u043e\u043a \u043d\u0435 \u0443\u043a\u0430\u0437\u0430\u043d"

    return-object p1

    .line 95
    :cond_1
    const-string v0, "kind"

    invoke-virtual {p1, v0}, Lorg/json/JSONObject;->optString(Ljava/lang/String;)Ljava/lang/String;

    move-result-object v0

    const-string v1, "unlimited"

    invoke-virtual {v1, v0}, Ljava/lang/String;->equals(Ljava/lang/Object;)Z

    move-result v0

    if-eqz v0, :cond_2

    const-string p1, "\u0411\u0435\u0437\u043b\u0438\u043c\u0438\u0442"

    return-object p1

    .line 96
    :cond_2
    new-instance v0, Ljava/util/Locale;

    const-string v1, "ru"

    const-string v2, "RU"

    invoke-direct {v0, v1, v2}, Ljava/util/Locale;-><init>(Ljava/lang/String;Ljava/lang/String;)V

    const/4 v1, 0x1

    invoke-static {v1, v0}, Ljava/text/DateFormat;->getDateInstance(ILjava/util/Locale;)Ljava/text/DateFormat;

    move-result-object v0

    new-instance v1, Ljava/util/Date;

    const-string v2, "expires_at"

    invoke-virtual {p1, v2}, Lorg/json/JSONObject;->optLong(Ljava/lang/String;)J

    move-result-wide v2

    const-wide/16 v4, 0x3e8

    mul-long/2addr v2, v4

    invoke-direct {v1, v2, v3}, Ljava/util/Date;-><init>(J)V

    invoke-virtual {v0, v1}, Ljava/text/DateFormat;->format(Ljava/util/Date;)Ljava/lang/String;

    move-result-object p1

    new-instance v0, Ljava/lang/StringBuilder;

    invoke-direct {v0}, Ljava/lang/StringBuilder;-><init>()V

    const-string v1, "\u0414\u043e "

    invoke-virtual {v0, v1}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v0

    invoke-virtual {v0, p1}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object p1

    invoke-virtual {p1}, Ljava/lang/StringBuilder;->toString()Ljava/lang/String;

    move-result-object p1

    return-object p1
.end method


# virtual methods
.method protected attachBaseContext(Landroid/content/Context;)V
    .locals 4

    .line 21
    new-instance v0, Landroid/content/res/Configuration;

    invoke-virtual {p1}, Landroid/content/Context;->getResources()Landroid/content/res/Resources;

    move-result-object v1

    invoke-virtual {v1}, Landroid/content/res/Resources;->getConfiguration()Landroid/content/res/Configuration;

    move-result-object v1

    invoke-direct {v0, v1}, Landroid/content/res/Configuration;-><init>(Landroid/content/res/Configuration;)V

    .line 22
    new-instance v1, Ljava/util/Locale;

    const-string v2, "ru"

    const-string v3, "RU"

    invoke-direct {v1, v2, v3}, Ljava/util/Locale;-><init>(Ljava/lang/String;Ljava/lang/String;)V

    .line 23
    invoke-static {v1}, Ljava/util/Locale;->setDefault(Ljava/util/Locale;)V

    .line 24
    invoke-virtual {v0, v1}, Landroid/content/res/Configuration;->setLocale(Ljava/util/Locale;)V

    .line 25
    invoke-virtual {p1, v0}, Landroid/content/Context;->createConfigurationContext(Landroid/content/res/Configuration;)Landroid/content/Context;

    move-result-object p1

    invoke-super {p0, p1}, Landroid/app/Activity;->attachBaseContext(Landroid/content/Context;)V

    .line 26
    return-void
.end method

.method synthetic lambda$chooseApps$24$org-garlic-client-MainActivity(Ljava/util/Set;Landroid/content/DialogInterface;I)V
    .locals 0

    .line 123
    iget-object p2, p0, Lorg/garlic/client/MainActivity;->prefs:Landroid/content/SharedPreferences;

    invoke-interface {p2}, Landroid/content/SharedPreferences;->edit()Landroid/content/SharedPreferences$Editor;

    move-result-object p2

    const-string p3, "bypass_packages"

    invoke-interface {p2, p3, p1}, Landroid/content/SharedPreferences$Editor;->putStringSet(Ljava/lang/String;Ljava/util/Set;)Landroid/content/SharedPreferences$Editor;

    move-result-object p1

    invoke-interface {p1}, Landroid/content/SharedPreferences$Editor;->apply()V

    iget-object p1, p0, Lorg/garlic/client/MainActivity;->message:Landroid/widget/TextView;

    const-string p2, "\u0421\u043f\u0438\u0441\u043e\u043a \u0441\u043e\u0445\u0440\u0430\u043d\u0451\u043d. \u041f\u0435\u0440\u0435\u043f\u043e\u0434\u043a\u043b\u044e\u0447\u0438\u0442\u0435\u0441\u044c \u0434\u043b\u044f \u043f\u0440\u0438\u043c\u0435\u043d\u0435\u043d\u0438\u044f"

    invoke-virtual {p1, p2}, Landroid/widget/TextView;->setText(Ljava/lang/CharSequence;)V

    return-void
.end method

.method synthetic lambda$editServer$32$org-garlic-client-MainActivity(Lorg/json/JSONObject;)V
    .locals 2

    .line 153
    const-string v0, "ok"

    invoke-virtual {p1, v0}, Lorg/json/JSONObject;->optBoolean(Ljava/lang/String;)Z

    move-result v0

    if-eqz v0, :cond_0

    invoke-direct {p0}, Lorg/garlic/client/MainActivity;->showServers()V

    iget-object p1, p0, Lorg/garlic/client/MainActivity;->message:Landroid/widget/TextView;

    const-string v0, "\u041b\u0438\u043c\u0438\u0442\u044b \u0441\u043e\u0445\u0440\u0430\u043d\u0435\u043d\u044b. \u041f\u0440\u0438\u043c\u0435\u043d\u0435\u043d\u0438\u0435 \u0432 \u0442\u0435\u0447\u0435\u043d\u0438\u0435 \u043d\u0435\u0441\u043a\u043e\u043b\u044c\u043a\u0438\u0445 \u0441\u0435\u043a\u0443\u043d\u0434"

    invoke-virtual {p1, v0}, Landroid/widget/TextView;->setText(Ljava/lang/CharSequence;)V

    goto :goto_0

    :cond_0
    iget-object v0, p0, Lorg/garlic/client/MainActivity;->message:Landroid/widget/TextView;

    const-string v1, "error"

    invoke-virtual {p1, v1}, Lorg/json/JSONObject;->optString(Ljava/lang/String;)Ljava/lang/String;

    move-result-object p1

    invoke-virtual {v0, p1}, Landroid/widget/TextView;->setText(Ljava/lang/CharSequence;)V

    :goto_0
    return-void
.end method

.method synthetic lambda$editServer$33$org-garlic-client-MainActivity()V
    .locals 2

    .line 153
    iget-object v0, p0, Lorg/garlic/client/MainActivity;->message:Landroid/widget/TextView;

    const-string v1, "\u041d\u0435 \u0443\u0434\u0430\u043b\u043e\u0441\u044c \u0441\u043e\u0445\u0440\u0430\u043d\u0438\u0442\u044c \u043b\u0438\u043c\u0438\u0442\u044b"

    invoke-virtual {v0, v1}, Landroid/widget/TextView;->setText(Ljava/lang/CharSequence;)V

    return-void
.end method

.method synthetic lambda$editServer$34$org-garlic-client-MainActivity(Lorg/json/JSONObject;)V
    .locals 3

    .line 153
    :try_start_0
    new-instance v0, Lorg/json/JSONObject;

    invoke-direct {p0}, Lorg/garlic/client/MainActivity;->dir()Ljava/io/File;

    move-result-object v1

    invoke-virtual {v1}, Ljava/io/File;->getAbsolutePath()Ljava/lang/String;

    move-result-object v1

    invoke-direct {p0}, Lorg/garlic/client/MainActivity;->origins()Ljava/lang/String;

    move-result-object v2

    invoke-virtual {p1}, Lorg/json/JSONObject;->toString()Ljava/lang/String;

    move-result-object p1

    invoke-static {v1, v2, p1}, Lorg/garlic/client/GarlicBridge;->adminNetwork(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;

    move-result-object p1

    invoke-direct {v0, p1}, Lorg/json/JSONObject;-><init>(Ljava/lang/String;)V

    new-instance p1, Lorg/garlic/client/MainActivity$0;

    invoke-direct {p1, p0, v0}, Lorg/garlic/client/MainActivity$0;-><init>(Lorg/garlic/client/MainActivity;Lorg/json/JSONObject;)V

    invoke-virtual {p0, p1}, Lorg/garlic/client/MainActivity;->runOnUiThread(Ljava/lang/Runnable;)V
    :try_end_0
    .catchall {:try_start_0 .. :try_end_0} :catchall_0

    goto :goto_0

    :catchall_0
    move-exception p1

    new-instance p1, Lorg/garlic/client/MainActivity$2;

    invoke-direct {p1, p0}, Lorg/garlic/client/MainActivity$2;-><init>(Lorg/garlic/client/MainActivity;)V

    invoke-virtual {p0, p1}, Lorg/garlic/client/MainActivity;->runOnUiThread(Ljava/lang/Runnable;)V

    :goto_0
    return-void
.end method

.method synthetic lambda$editServer$35$org-garlic-client-MainActivity(Ljava/lang/String;Landroid/widget/EditText;Landroid/widget/EditText;Landroid/content/DialogInterface;I)V
    .locals 0

    .line 153
    :try_start_0
    new-instance p4, Lorg/json/JSONObject;

    invoke-direct {p4}, Lorg/json/JSONObject;-><init>()V

    const-string p5, "node_id"

    invoke-virtual {p4, p5, p1}, Lorg/json/JSONObject;->put(Ljava/lang/String;Ljava/lang/Object;)Lorg/json/JSONObject;

    const-string p1, "stream_limit"

    invoke-virtual {p2}, Landroid/widget/EditText;->getText()Landroid/text/Editable;

    move-result-object p2

    invoke-virtual {p2}, Ljava/lang/Object;->toString()Ljava/lang/String;

    move-result-object p2

    invoke-static {p2}, Ljava/lang/Integer;->parseInt(Ljava/lang/String;)I

    move-result p2

    invoke-virtual {p4, p1, p2}, Lorg/json/JSONObject;->put(Ljava/lang/String;I)Lorg/json/JSONObject;

    const-string p1, "worker_limit"

    invoke-virtual {p3}, Landroid/widget/EditText;->getText()Landroid/text/Editable;

    move-result-object p2

    invoke-virtual {p2}, Ljava/lang/Object;->toString()Ljava/lang/String;

    move-result-object p2

    invoke-static {p2}, Ljava/lang/Integer;->parseInt(Ljava/lang/String;)I

    move-result p2

    invoke-virtual {p4, p1, p2}, Lorg/json/JSONObject;->put(Ljava/lang/String;I)Lorg/json/JSONObject;

    iget-object p1, p0, Lorg/garlic/client/MainActivity;->worker:Ljava/util/concurrent/ExecutorService;

    new-instance p2, Lorg/garlic/client/MainActivity$44;

    invoke-direct {p2, p0, p4}, Lorg/garlic/client/MainActivity$44;-><init>(Lorg/garlic/client/MainActivity;Lorg/json/JSONObject;)V

    invoke-interface {p1, p2}, Ljava/util/concurrent/ExecutorService;->execute(Ljava/lang/Runnable;)V
    :try_end_0
    .catch Ljava/lang/Exception; {:try_start_0 .. :try_end_0} :catch_0

    goto :goto_0

    :catch_0
    move-exception p1

    iget-object p1, p0, Lorg/garlic/client/MainActivity;->message:Landroid/widget/TextView;

    const-string p2, "\u0412\u0432\u0435\u0434\u0438\u0442\u0435 \u0447\u0438\u0441\u043b\u0430 \u043e\u0442 1 \u0434\u043e 4096"

    invoke-virtual {p1, p2}, Landroid/widget/TextView;->setText(Ljava/lang/CharSequence;)V

    .line 154
    :goto_0
    return-void
.end method

.method synthetic lambda$login$3$org-garlic-client-MainActivity(ILandroid/widget/Button;Lorg/json/JSONObject;Ljava/lang/String;)V
    .locals 2

    .line 88
    const-string v0, "direct_urls"

    iget v1, p0, Lorg/garlic/client/MainActivity;->generation:I

    if-eq p1, v1, :cond_0

    return-void

    :cond_0
    const/4 p1, 0x1

    invoke-virtual {p2, p1}, Landroid/widget/Button;->setEnabled(Z)V

    :try_start_0
    const-string p2, "ok"

    invoke-virtual {p3, p2}, Lorg/json/JSONObject;->optBoolean(Ljava/lang/String;)Z

    move-result p2

    if-nez p2, :cond_1

    iget-object p1, p0, Lorg/garlic/client/MainActivity;->message:Landroid/widget/TextView;

    const-string p2, "error"

    const-string p4, "\u041d\u0435 \u0443\u0434\u0430\u043b\u043e\u0441\u044c \u0432\u043e\u0439\u0442\u0438"

    invoke-virtual {p3, p2, p4}, Lorg/json/JSONObject;->optString(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;

    move-result-object p2

    invoke-virtual {p1, p2}, Landroid/widget/TextView;->setText(Ljava/lang/CharSequence;)V

    return-void

    :cond_1
    iget-object p2, p0, Lorg/garlic/client/MainActivity;->prefs:Landroid/content/SharedPreferences;

    invoke-interface {p2}, Landroid/content/SharedPreferences;->edit()Landroid/content/SharedPreferences$Editor;

    move-result-object p2

    const-string v1, "bootstrap"

    invoke-interface {p2, v1, p4}, Landroid/content/SharedPreferences$Editor;->putString(Ljava/lang/String;Ljava/lang/String;)Landroid/content/SharedPreferences$Editor;

    move-result-object p2

    const-string p4, "mode"

    const-string v1, "auto"

    invoke-interface {p2, p4, v1}, Landroid/content/SharedPreferences$Editor;->putString(Ljava/lang/String;Ljava/lang/String;)Landroid/content/SharedPreferences$Editor;

    move-result-object p2

    invoke-virtual {p3, v0}, Lorg/json/JSONObject;->getJSONArray(Ljava/lang/String;)Lorg/json/JSONArray;

    move-result-object p4

    invoke-direct {p0, p4}, Lorg/garlic/client/MainActivity;->urls(Lorg/json/JSONArray;)Ljava/lang/String;

    move-result-object p4

    invoke-interface {p2, v0, p4}, Landroid/content/SharedPreferences$Editor;->putString(Ljava/lang/String;Ljava/lang/String;)Landroid/content/SharedPreferences$Editor;

    move-result-object p2

    const-string p4, "ru_urls"

    const-string v0, "control_urls"

    invoke-virtual {p3, v0}, Lorg/json/JSONObject;->getJSONArray(Ljava/lang/String;)Lorg/json/JSONArray;

    move-result-object v0

    invoke-direct {p0, v0}, Lorg/garlic/client/MainActivity;->urls(Lorg/json/JSONArray;)Ljava/lang/String;

    move-result-object v0

    invoke-interface {p2, p4, v0}, Landroid/content/SharedPreferences$Editor;->putString(Ljava/lang/String;Ljava/lang/String;)Landroid/content/SharedPreferences$Editor;

    move-result-object p2

    const-string p4, "key"

    invoke-interface {p2, p4}, Landroid/content/SharedPreferences$Editor;->remove(Ljava/lang/String;)Landroid/content/SharedPreferences$Editor;

    move-result-object p2

    const-string p4, "password"

    invoke-interface {p2, p4}, Landroid/content/SharedPreferences$Editor;->remove(Ljava/lang/String;)Landroid/content/SharedPreferences$Editor;

    move-result-object p2

    invoke-interface {p2}, Landroid/content/SharedPreferences$Editor;->apply()V

    const-string p2, "account"

    invoke-virtual {p3, p2}, Lorg/json/JSONObject;->getJSONObject(Ljava/lang/String;)Lorg/json/JSONObject;

    move-result-object p2

    iput-object p2, p0, Lorg/garlic/client/MainActivity;->account:Lorg/json/JSONObject;

    iget-object p2, p0, Lorg/garlic/client/MainActivity;->prefs:Landroid/content/SharedPreferences;

    invoke-interface {p2}, Landroid/content/SharedPreferences;->edit()Landroid/content/SharedPreferences$Editor;

    move-result-object p2

    const-string p3, "account_cache"

    iget-object p4, p0, Lorg/garlic/client/MainActivity;->account:Lorg/json/JSONObject;

    invoke-virtual {p4}, Lorg/json/JSONObject;->toString()Ljava/lang/String;

    move-result-object p4

    invoke-interface {p2, p3, p4}, Landroid/content/SharedPreferences$Editor;->putString(Ljava/lang/String;Ljava/lang/String;)Landroid/content/SharedPreferences$Editor;

    move-result-object p2

    invoke-interface {p2}, Landroid/content/SharedPreferences$Editor;->apply()V

    iput-boolean p1, p0, Lorg/garlic/client/MainActivity;->roleVerified:Z

    iget-object p2, p0, Lorg/garlic/client/MainActivity;->key:Landroid/widget/EditText;

    const-string p3, ""

    invoke-virtual {p2, p3}, Landroid/widget/EditText;->setText(Ljava/lang/CharSequence;)V

    iput-boolean p1, p0, Lorg/garlic/client/MainActivity;->loggedIn:Z

    invoke-static {p0}, Lorg/garlic/client/CoreLoader;->check(Landroid/content/Context;)V

    invoke-direct {p0}, Lorg/garlic/client/MainActivity;->showHome()V
    :try_end_0
    .catch Ljava/lang/Exception; {:try_start_0 .. :try_end_0} :catch_0

    goto :goto_0

    :catch_0
    move-exception p1

    iget-object p1, p0, Lorg/garlic/client/MainActivity;->message:Landroid/widget/TextView;

    const-string p2, "\u041d\u0435\u043a\u043e\u0440\u0440\u0435\u043a\u0442\u043d\u044b\u0439 \u043e\u0442\u0432\u0435\u0442 \u0441\u0435\u0440\u0432\u0435\u0440\u0430"

    invoke-virtual {p1, p2}, Landroid/widget/TextView;->setText(Ljava/lang/CharSequence;)V

    :goto_0
    return-void
.end method

.method synthetic lambda$login$4$org-garlic-client-MainActivity(ILandroid/widget/Button;)V
    .locals 1

    .line 88
    iget v0, p0, Lorg/garlic/client/MainActivity;->generation:I

    if-ne p1, v0, :cond_0

    const/4 p1, 0x1

    invoke-virtual {p2, p1}, Landroid/widget/Button;->setEnabled(Z)V

    iget-object p1, p0, Lorg/garlic/client/MainActivity;->message:Landroid/widget/TextView;

    const-string p2, "\u041d\u0435 \u0443\u0434\u0430\u043b\u043e\u0441\u044c \u043f\u043e\u0434\u043a\u043b\u044e\u0447\u0438\u0442\u044c\u0441\u044f \u043a \u0441\u0435\u0440\u0432\u0435\u0440\u0443"

    invoke-virtual {p1, p2}, Landroid/widget/TextView;->setText(Ljava/lang/CharSequence;)V

    :cond_0
    return-void
.end method

.method synthetic lambda$login$5$org-garlic-client-MainActivity(Ljava/lang/String;Ljava/lang/String;ILandroid/widget/Button;)V
    .locals 6

    .line 88
    :try_start_0
    new-instance v4, Lorg/json/JSONObject;

    invoke-direct {p0}, Lorg/garlic/client/MainActivity;->dir()Ljava/io/File;

    move-result-object v0

    invoke-virtual {v0}, Ljava/io/File;->getAbsolutePath()Ljava/lang/String;

    move-result-object v0

    invoke-static {v0, p1, p2}, Lorg/garlic/client/GarlicBridge;->connectNetwork(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;

    move-result-object p2

    invoke-direct {v4, p2}, Lorg/json/JSONObject;-><init>(Ljava/lang/String;)V

    new-instance v0, Lorg/garlic/client/MainActivity$18;
    :try_end_0
    .catchall {:try_start_0 .. :try_end_0} :catchall_1

    move-object v1, p0

    move-object v5, p1

    move v2, p3

    move-object v3, p4

    :try_start_1
    invoke-direct/range {v0 .. v5}, Lorg/garlic/client/MainActivity$18;-><init>(Lorg/garlic/client/MainActivity;ILandroid/widget/Button;Lorg/json/JSONObject;Ljava/lang/String;)V

    invoke-virtual {p0, v0}, Lorg/garlic/client/MainActivity;->runOnUiThread(Ljava/lang/Runnable;)V
    :try_end_1
    .catchall {:try_start_1 .. :try_end_1} :catchall_0

    goto :goto_1

    :catchall_0
    move-exception v0

    goto :goto_0

    :catchall_1
    move-exception v0

    move v2, p3

    move-object v3, p4

    :goto_0
    new-instance p1, Lorg/garlic/client/MainActivity$19;

    invoke-direct {p1, p0, v2, v3}, Lorg/garlic/client/MainActivity$19;-><init>(Lorg/garlic/client/MainActivity;ILandroid/widget/Button;)V

    invoke-virtual {p0, p1}, Lorg/garlic/client/MainActivity;->runOnUiThread(Ljava/lang/Runnable;)V

    :goto_1
    return-void
.end method

.method synthetic lambda$logout$21$org-garlic-client-MainActivity()V
    .locals 2

    .line 117
    iget-object v0, p0, Lorg/garlic/client/MainActivity;->prefs:Landroid/content/SharedPreferences;

    invoke-interface {v0}, Landroid/content/SharedPreferences;->edit()Landroid/content/SharedPreferences$Editor;

    move-result-object v0

    const-string v1, "key"

    invoke-interface {v0, v1}, Landroid/content/SharedPreferences$Editor;->remove(Ljava/lang/String;)Landroid/content/SharedPreferences$Editor;

    move-result-object v0

    const-string v1, "password"

    invoke-interface {v0, v1}, Landroid/content/SharedPreferences$Editor;->remove(Ljava/lang/String;)Landroid/content/SharedPreferences$Editor;

    move-result-object v0

    const-string v1, "direct_urls"

    invoke-interface {v0, v1}, Landroid/content/SharedPreferences$Editor;->remove(Ljava/lang/String;)Landroid/content/SharedPreferences$Editor;

    move-result-object v0

    const-string v1, "ru_urls"

    invoke-interface {v0, v1}, Landroid/content/SharedPreferences$Editor;->remove(Ljava/lang/String;)Landroid/content/SharedPreferences$Editor;

    move-result-object v0

    const-string v1, "account_cache"

    invoke-interface {v0, v1}, Landroid/content/SharedPreferences$Editor;->remove(Ljava/lang/String;)Landroid/content/SharedPreferences$Editor;

    move-result-object v0

    invoke-interface {v0}, Landroid/content/SharedPreferences$Editor;->apply()V

    const/4 v0, 0x0

    iput-boolean v0, p0, Lorg/garlic/client/MainActivity;->roleVerified:Z

    const/4 v0, 0x0

    iput-object v0, p0, Lorg/garlic/client/MainActivity;->account:Lorg/json/JSONObject;

    invoke-direct {p0}, Lorg/garlic/client/MainActivity;->showLogin()V

    return-void
.end method

.method synthetic lambda$logout$22$org-garlic-client-MainActivity()V
    .locals 5

    .line 117
    :try_start_0
    invoke-static {}, Lorg/garlic/client/GarlicBridge;->stopProxy()Ljava/lang/String;
    :try_end_0
    .catchall {:try_start_0 .. :try_end_0} :catchall_0

    goto :goto_0

    :catchall_0
    move-exception v0

    :goto_0
    const-string v0, "client.preferences.json"

    const-string v1, "client.account-cache.json"

    const-string v2, "client.sessions.json"

    const-string v3, "client.device.json"

    const-string v4, "client.network.json"

    filled-new-array {v2, v3, v4, v0, v1}, [Ljava/lang/String;

    move-result-object v0

    const/4 v1, 0x0

    :goto_1
    const/4 v2, 0x5

    if-ge v1, v2, :cond_0

    aget-object v2, v0, v1

    new-instance v3, Ljava/io/File;

    invoke-direct {p0}, Lorg/garlic/client/MainActivity;->dir()Ljava/io/File;

    move-result-object v4

    invoke-direct {v3, v4, v2}, Ljava/io/File;-><init>(Ljava/io/File;Ljava/lang/String;)V

    invoke-virtual {v3}, Ljava/io/File;->delete()Z

    add-int/lit8 v1, v1, 0x1

    goto :goto_1

    :cond_0
    new-instance v0, Lorg/garlic/client/MainActivity$39;

    invoke-direct {v0, p0}, Lorg/garlic/client/MainActivity$39;-><init>(Lorg/garlic/client/MainActivity;)V

    invoke-virtual {p0, v0}, Lorg/garlic/client/MainActivity;->runOnUiThread(Ljava/lang/Runnable;)V

    return-void
.end method

.method synthetic lambda$manage$49$org-garlic-client-MainActivity(Ljava/lang/String;Landroid/content/DialogInterface;I)V
    .locals 0

    .line 182
    const-string p2, "clipboard"

    invoke-virtual {p0, p2}, Lorg/garlic/client/MainActivity;->getSystemService(Ljava/lang/String;)Ljava/lang/Object;

    move-result-object p2

    check-cast p2, Landroid/content/ClipboardManager;

    const-string p3, "Garlic"

    invoke-static {p3, p1}, Landroid/content/ClipData;->newPlainText(Ljava/lang/CharSequence;Ljava/lang/CharSequence;)Landroid/content/ClipData;

    move-result-object p1

    invoke-virtual {p2, p1}, Landroid/content/ClipboardManager;->setPrimaryClip(Landroid/content/ClipData;)V

    return-void
.end method

.method synthetic lambda$manage$50$org-garlic-client-MainActivity(ILorg/json/JSONObject;)V
    .locals 2

    .line 182
    iget v0, p0, Lorg/garlic/client/MainActivity;->generation:I

    if-ne p1, v0, :cond_2

    iget-boolean p1, p0, Lorg/garlic/client/MainActivity;->admin:Z

    if-nez p1, :cond_0

    goto :goto_0

    :cond_0
    const-string p1, "error"

    invoke-virtual {p2, p1}, Lorg/json/JSONObject;->has(Ljava/lang/String;)Z

    move-result v0

    if-eqz v0, :cond_1

    iget-object v0, p0, Lorg/garlic/client/MainActivity;->message:Landroid/widget/TextView;

    invoke-virtual {p2, p1}, Lorg/json/JSONObject;->optString(Ljava/lang/String;)Ljava/lang/String;

    move-result-object p1

    invoke-virtual {v0, p1}, Landroid/widget/TextView;->setText(Ljava/lang/CharSequence;)V

    return-void

    :cond_1
    const-string p1, "token"

    const-string v0, ""

    invoke-virtual {p2, p1, v0}, Lorg/json/JSONObject;->optString(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;

    move-result-object p1

    invoke-direct {p0}, Lorg/garlic/client/MainActivity;->showUsers()V

    invoke-virtual {p1}, Ljava/lang/String;->isEmpty()Z

    move-result p2

    if-nez p2, :cond_2

    new-instance p2, Landroid/app/AlertDialog$Builder;

    invoke-direct {p2, p0}, Landroid/app/AlertDialog$Builder;-><init>(Landroid/content/Context;)V

    const-string v0, "\u041a\u043b\u044e\u0447 \u043f\u043e\u043b\u044c\u0437\u043e\u0432\u0430\u0442\u0435\u043b\u044f"

    invoke-virtual {p2, v0}, Landroid/app/AlertDialog$Builder;->setTitle(Ljava/lang/CharSequence;)Landroid/app/AlertDialog$Builder;

    move-result-object p2

    invoke-virtual {p2, p1}, Landroid/app/AlertDialog$Builder;->setMessage(Ljava/lang/CharSequence;)Landroid/app/AlertDialog$Builder;

    move-result-object p2

    const-string v0, "\u0417\u0430\u043a\u0440\u044b\u0442\u044c"

    const/4 v1, 0x0

    invoke-virtual {p2, v0, v1}, Landroid/app/AlertDialog$Builder;->setNegativeButton(Ljava/lang/CharSequence;Landroid/content/DialogInterface$OnClickListener;)Landroid/app/AlertDialog$Builder;

    move-result-object p2

    new-instance v0, Lorg/garlic/client/MainActivity$14;

    invoke-direct {v0, p0, p1}, Lorg/garlic/client/MainActivity$14;-><init>(Lorg/garlic/client/MainActivity;Ljava/lang/String;)V

    const-string p1, "\u0421\u043a\u043e\u043f\u0438\u0440\u043e\u0432\u0430\u0442\u044c"

    invoke-virtual {p2, p1, v0}, Landroid/app/AlertDialog$Builder;->setPositiveButton(Ljava/lang/CharSequence;Landroid/content/DialogInterface$OnClickListener;)Landroid/app/AlertDialog$Builder;

    move-result-object p1

    invoke-virtual {p1}, Landroid/app/AlertDialog$Builder;->show()Landroid/app/AlertDialog;

    :cond_2
    :goto_0
    return-void
.end method

.method synthetic lambda$manage$51$org-garlic-client-MainActivity()V
    .locals 2

    .line 182
    iget-object v0, p0, Lorg/garlic/client/MainActivity;->message:Landroid/widget/TextView;

    const-string v1, "\u041d\u0435 \u0443\u0434\u0430\u043b\u043e\u0441\u044c \u0441\u043e\u0445\u0440\u0430\u043d\u0438\u0442\u044c \u0438\u0437\u043c\u0435\u043d\u0435\u043d\u0438\u044f"

    invoke-virtual {v0, v1}, Landroid/widget/TextView;->setText(Ljava/lang/CharSequence;)V

    return-void
.end method

.method synthetic lambda$manage$52$org-garlic-client-MainActivity(Lorg/json/JSONObject;I)V
    .locals 3

    .line 182
    :try_start_0
    new-instance v0, Lorg/json/JSONObject;

    invoke-direct {p0}, Lorg/garlic/client/MainActivity;->dir()Ljava/io/File;

    move-result-object v1

    invoke-virtual {v1}, Ljava/io/File;->getAbsolutePath()Ljava/lang/String;

    move-result-object v1

    invoke-direct {p0}, Lorg/garlic/client/MainActivity;->origins()Ljava/lang/String;

    move-result-object v2

    invoke-virtual {p1}, Lorg/json/JSONObject;->toString()Ljava/lang/String;

    move-result-object p1

    invoke-static {v1, v2, p1}, Lorg/garlic/client/GarlicBridge;->adminManageUser(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;

    move-result-object p1

    invoke-direct {v0, p1}, Lorg/json/JSONObject;-><init>(Ljava/lang/String;)V

    new-instance p1, Lorg/garlic/client/MainActivity$34;

    invoke-direct {p1, p0, p2, v0}, Lorg/garlic/client/MainActivity$34;-><init>(Lorg/garlic/client/MainActivity;ILorg/json/JSONObject;)V

    invoke-virtual {p0, p1}, Lorg/garlic/client/MainActivity;->runOnUiThread(Ljava/lang/Runnable;)V
    :try_end_0
    .catchall {:try_start_0 .. :try_end_0} :catchall_0

    goto :goto_0

    :catchall_0
    move-exception p1

    new-instance p1, Lorg/garlic/client/MainActivity$35;

    invoke-direct {p1, p0}, Lorg/garlic/client/MainActivity$35;-><init>(Lorg/garlic/client/MainActivity;)V

    invoke-virtual {p0, p1}, Lorg/garlic/client/MainActivity;->runOnUiThread(Ljava/lang/Runnable;)V

    :goto_0
    return-void
.end method

.method synthetic lambda$refreshAccount$6$org-garlic-client-MainActivity(ILorg/json/JSONObject;)V
    .locals 1

    .line 90
    iget v0, p0, Lorg/garlic/client/MainActivity;->generation:I

    if-eq p1, v0, :cond_0

    return-void

    :cond_0
    const-string p1, "ok"

    invoke-virtual {p2, p1}, Lorg/json/JSONObject;->optBoolean(Ljava/lang/String;)Z

    move-result p1

    if-eqz p1, :cond_1

    const-string p1, "account"

    invoke-virtual {p2, p1}, Lorg/json/JSONObject;->optJSONObject(Ljava/lang/String;)Lorg/json/JSONObject;

    move-result-object p1

    iput-object p1, p0, Lorg/garlic/client/MainActivity;->account:Lorg/json/JSONObject;

    iget-object p1, p0, Lorg/garlic/client/MainActivity;->prefs:Landroid/content/SharedPreferences;

    invoke-interface {p1}, Landroid/content/SharedPreferences;->edit()Landroid/content/SharedPreferences$Editor;

    move-result-object p1

    iget-object p2, p0, Lorg/garlic/client/MainActivity;->account:Lorg/json/JSONObject;

    invoke-virtual {p2}, Lorg/json/JSONObject;->toString()Ljava/lang/String;

    move-result-object p2

    const-string v0, "account_cache"

    invoke-interface {p1, v0, p2}, Landroid/content/SharedPreferences$Editor;->putString(Ljava/lang/String;Ljava/lang/String;)Landroid/content/SharedPreferences$Editor;

    move-result-object p1

    invoke-interface {p1}, Landroid/content/SharedPreferences$Editor;->apply()V

    const/4 p1, 0x1

    iput-boolean p1, p0, Lorg/garlic/client/MainActivity;->roleVerified:Z

    iput-boolean p1, p0, Lorg/garlic/client/MainActivity;->loggedIn:Z

    invoke-direct {p0}, Lorg/garlic/client/MainActivity;->showHome()V

    :cond_1
    return-void
.end method

.method synthetic lambda$refreshAccount$7$org-garlic-client-MainActivity(I)V
    .locals 3

    .line 90
    :try_start_0
    new-instance v0, Lorg/json/JSONObject;

    invoke-direct {p0}, Lorg/garlic/client/MainActivity;->dir()Ljava/io/File;

    move-result-object v1

    invoke-virtual {v1}, Ljava/io/File;->getAbsolutePath()Ljava/lang/String;

    move-result-object v1

    invoke-direct {p0}, Lorg/garlic/client/MainActivity;->origins()Ljava/lang/String;

    move-result-object v2

    invoke-static {v1, v2}, Lorg/garlic/client/GarlicBridge;->account(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;

    move-result-object v1

    invoke-direct {v0, v1}, Lorg/json/JSONObject;-><init>(Ljava/lang/String;)V

    new-instance v1, Lorg/garlic/client/MainActivity$13;

    invoke-direct {v1, p0, p1, v0}, Lorg/garlic/client/MainActivity$13;-><init>(Lorg/garlic/client/MainActivity;ILorg/json/JSONObject;)V

    invoke-virtual {p0, v1}, Lorg/garlic/client/MainActivity;->runOnUiThread(Ljava/lang/Runnable;)V
    :try_end_0
    .catchall {:try_start_0 .. :try_end_0} :catchall_0

    goto :goto_0

    :catchall_0
    move-exception p1

    :goto_0
    return-void
.end method

.method synthetic lambda$shell$0$org-garlic-client-MainActivity(Landroid/content/DialogInterface;I)V
    .locals 0

    .line 68
    if-nez p2, :cond_0

    invoke-direct {p0}, Lorg/garlic/client/MainActivity;->showUsers()V

    goto :goto_0

    :cond_0
    const/4 p1, 0x1

    if-ne p2, p1, :cond_1

    invoke-direct {p0}, Lorg/garlic/client/MainActivity;->showServers()V

    goto :goto_0

    :cond_1
    invoke-direct {p0}, Lorg/garlic/client/MainActivity;->showLegacy()V

    :goto_0
    return-void
.end method

.method synthetic lambda$shell$1$org-garlic-client-MainActivity(Landroid/view/View;)V
    .locals 3

    .line 68
    new-instance p1, Landroid/app/AlertDialog$Builder;

    invoke-direct {p1, p0}, Landroid/app/AlertDialog$Builder;-><init>(Landroid/content/Context;)V

    const-string v0, "\u0421\u0435\u0440\u0432\u0435\u0440\u044b"

    const-string v1, "\u041f\u043e\u043b\u044c\u0437\u043e\u0432\u0430\u0442\u0435\u043b\u0438 Legacy"

    const-string v2, "\u041f\u043e\u043b\u044c\u0437\u043e\u0432\u0430\u0442\u0435\u043b\u0438"

    filled-new-array {v2, v0, v1}, [Ljava/lang/String;

    move-result-object v0

    new-instance v1, Lorg/garlic/client/MainActivity$41;

    invoke-direct {v1, p0}, Lorg/garlic/client/MainActivity$41;-><init>(Lorg/garlic/client/MainActivity;)V

    invoke-virtual {p1, v0, v1}, Landroid/app/AlertDialog$Builder;->setItems([Ljava/lang/CharSequence;Landroid/content/DialogInterface$OnClickListener;)Landroid/app/AlertDialog$Builder;

    move-result-object p1

    invoke-virtual {p1}, Landroid/app/AlertDialog$Builder;->show()Landroid/app/AlertDialog;

    return-void
.end method

.method synthetic lambda$showHome$10$org-garlic-client-MainActivity(Landroid/widget/Button;Landroid/view/View;)V
    .locals 3

    .line 101
    new-instance p2, Landroid/app/AlertDialog$Builder;

    invoke-direct {p2, p0}, Landroid/app/AlertDialog$Builder;-><init>(Landroid/content/Context;)V

    const-string v0, "\u041c\u0430\u0440\u0448\u0440\u0443\u0442"

    invoke-virtual {p2, v0}, Landroid/app/AlertDialog$Builder;->setTitle(Ljava/lang/CharSequence;)Landroid/app/AlertDialog$Builder;

    move-result-object p2

    const-string v0, "\u041d\u0430\u043f\u0440\u044f\u043c\u0443\u044e"

    const-string v1, "\u0427\u0435\u0440\u0435\u0437 \u0420\u043e\u0441\u0441\u0438\u044e"

    const-string v2, "\u0410\u0432\u0442\u043e\u043c\u0430\u0442\u0438\u0447\u0435\u0441\u043a\u0438"

    filled-new-array {v2, v0, v1}, [Ljava/lang/String;

    move-result-object v0

    new-instance v1, Lorg/garlic/client/MainActivity$16;

    invoke-direct {v1, p0, p1}, Lorg/garlic/client/MainActivity$16;-><init>(Lorg/garlic/client/MainActivity;Landroid/widget/Button;)V

    invoke-virtual {p2, v0, v1}, Landroid/app/AlertDialog$Builder;->setItems([Ljava/lang/CharSequence;Landroid/content/DialogInterface$OnClickListener;)Landroid/app/AlertDialog$Builder;

    move-result-object p1

    invoke-virtual {p1}, Landroid/app/AlertDialog$Builder;->show()Landroid/app/AlertDialog;

    return-void
.end method

.method synthetic lambda$showHome$11$org-garlic-client-MainActivity(Landroid/content/DialogInterface;I)V
    .locals 2

    .line 102
    if-nez p2, :cond_0

    iget-object p1, p0, Lorg/garlic/client/MainActivity;->prefs:Landroid/content/SharedPreferences;

    invoke-interface {p1}, Landroid/content/SharedPreferences;->edit()Landroid/content/SharedPreferences$Editor;

    move-result-object p1

    iget-object p2, p0, Lorg/garlic/client/MainActivity;->prefs:Landroid/content/SharedPreferences;

    const-string v0, "show_statistics"

    const/4 v1, 0x1

    invoke-interface {p2, v0, v1}, Landroid/content/SharedPreferences;->getBoolean(Ljava/lang/String;Z)Z

    move-result p2

    xor-int/2addr p2, v1

    invoke-interface {p1, v0, p2}, Landroid/content/SharedPreferences$Editor;->putBoolean(Ljava/lang/String;Z)Landroid/content/SharedPreferences$Editor;

    move-result-object p1

    invoke-interface {p1}, Landroid/content/SharedPreferences$Editor;->apply()V

    goto :goto_0

    :cond_0
    const/4 v0, 0x2
    if-ne p2, v0, :check_protocols
    invoke-static {p0}, Lorg/garlic/client/BackgroundSettings;->show(Landroid/app/Activity;)V
    return-void
    :check_protocols
    const/4 v0, 0x3
    if-ne p2, v0, :choose_apps
    invoke-static {p0}, Lorg/garlic/client/BackgroundSettings;->chooseProtocols(Landroid/app/Activity;)V
    return-void
    :choose_apps
    const/4 v0, 0x4
    if-ne p2, v0, :alpha_apps
    invoke-static {p0}, Lorg/garlic/client/BackgroundSettings;->exportLogs(Landroid/app/Activity;)V
    return-void
    :alpha_apps
    const/4 v0, 0x5
    if-ne p2, v0, :pick_apps
    invoke-static {p0}, Lorg/garlic/client/ReleaseUpdate;->channel(Landroid/app/Activity;)V
    return-void
    :pick_apps
    invoke-direct {p0}, Lorg/garlic/client/MainActivity;->chooseApps()V

    :goto_0
    return-void
.end method

.method synthetic lambda$showHome$12$org-garlic-client-MainActivity(Landroid/view/View;)V
    .locals 5

    .line 102
    new-instance p1, Landroid/app/AlertDialog$Builder;

    invoke-direct {p1, p0}, Landroid/app/AlertDialog$Builder;-><init>(Landroid/content/Context;)V

    iget-object v0, p0, Lorg/garlic/client/MainActivity;->prefs:Landroid/content/SharedPreferences;

    const-string v1, "show_statistics"

    const/4 v2, 0x1

    invoke-interface {v0, v1, v2}, Landroid/content/SharedPreferences;->getBoolean(Ljava/lang/String;Z)Z

    move-result v0

    if-eqz v0, :cond_0

    const-string v0, "\u0421\u043a\u0440\u044b\u0442\u044c \u0441\u043a\u043e\u0440\u043e\u0441\u0442\u044c \u0438 \u043e\u0442\u043a\u043b\u0438\u043a"

    goto :goto_0

    :cond_0
    const-string v0, "\u041f\u043e\u043a\u0430\u0437\u044b\u0432\u0430\u0442\u044c \u0441\u043a\u043e\u0440\u043e\u0441\u0442\u044c \u0438 \u043e\u0442\u043a\u043b\u0438\u043a"

    :goto_0
    const-string v1, "\u041f\u0440\u0438\u043b\u043e\u0436\u0435\u043d\u0438\u044f \u0432\u043d\u0435 Garlic"

    const-string v2, "Работа в фоне"

    const-string v3, "Протоколы"

    const-string v4, "Экспорт журнала"

    filled-new-array {v0, v1, v2, v3, v4}, [Ljava/lang/String;

    move-result-object v0

    invoke-static {v0}, Lorg/garlic/client/ReleaseUpdate;->withChannel([Ljava/lang/String;)[Ljava/lang/String;
    move-result-object v0

    new-instance v1, Lorg/garlic/client/MainActivity$40;

    invoke-direct {v1, p0}, Lorg/garlic/client/MainActivity$40;-><init>(Lorg/garlic/client/MainActivity;)V

    invoke-virtual {p1, v0, v1}, Landroid/app/AlertDialog$Builder;->setItems([Ljava/lang/CharSequence;Landroid/content/DialogInterface$OnClickListener;)Landroid/app/AlertDialog$Builder;

    move-result-object p1

    invoke-virtual {p1}, Landroid/app/AlertDialog$Builder;->show()Landroid/app/AlertDialog;

    return-void
.end method

.method synthetic lambda$showHome$13$org-garlic-client-MainActivity(Landroid/view/View;)V
    .locals 0

    .line 104
    invoke-direct {p0}, Lorg/garlic/client/MainActivity;->takeover()V

    return-void
.end method

.method synthetic lambda$showHome$14$org-garlic-client-MainActivity(Landroid/view/View;)V
    .locals 0

    .line 105
    invoke-direct {p0}, Lorg/garlic/client/MainActivity;->showUsers()V

    return-void
.end method

.method synthetic lambda$showHome$15$org-garlic-client-MainActivity(Landroid/view/View;)V
    .locals 0

    .line 105
    invoke-direct {p0}, Lorg/garlic/client/MainActivity;->showServers()V

    return-void
.end method

.method synthetic lambda$showHome$16$org-garlic-client-MainActivity(Landroid/view/View;)V
    .locals 0

    .line 105
    invoke-direct {p0}, Lorg/garlic/client/MainActivity;->showLegacy()V

    return-void
.end method

.method synthetic lambda$showHome$17$org-garlic-client-MainActivity(Landroid/view/View;)V
    .locals 0

    .line 106
    invoke-direct {p0}, Lorg/garlic/client/MainActivity;->logout()V

    return-void
.end method

.method synthetic lambda$showHome$8$org-garlic-client-MainActivity(Landroid/view/View;)V
    .locals 0

    .line 100
    sget-boolean p1, Lorg/garlic/client/GarlicProxyService;->isRunning:Z

    if-nez p1, :cond_1

    sget-boolean p1, Lorg/garlic/client/GarlicProxyService;->isConnecting:Z

    if-eqz p1, :cond_0

    goto :goto_0

    :cond_0
    invoke-direct {p0}, Lorg/garlic/client/MainActivity;->startVpn()V

    goto :goto_1

    :cond_1
    :goto_0
    invoke-direct {p0}, Lorg/garlic/client/MainActivity;->stopVpn()V

    :goto_1
    return-void
.end method

.method synthetic lambda$showHome$9$org-garlic-client-MainActivity(Landroid/widget/Button;Landroid/content/DialogInterface;I)V
    .locals 3

    .line 101
    iget-object p2, p0, Lorg/garlic/client/MainActivity;->prefs:Landroid/content/SharedPreferences;

    invoke-interface {p2}, Landroid/content/SharedPreferences;->edit()Landroid/content/SharedPreferences$Editor;

    move-result-object p2

    const-string v0, "direct"

    const-string v1, "ru"

    const-string v2, "auto"

    filled-new-array {v2, v0, v1}, [Ljava/lang/String;

    move-result-object v0

    aget-object p3, v0, p3

    const-string v0, "mode"

    invoke-interface {p2, v0, p3}, Landroid/content/SharedPreferences$Editor;->putString(Ljava/lang/String;Ljava/lang/String;)Landroid/content/SharedPreferences$Editor;

    move-result-object p2

    invoke-interface {p2}, Landroid/content/SharedPreferences$Editor;->apply()V

    invoke-direct {p0}, Lorg/garlic/client/MainActivity;->routeName()Ljava/lang/String;

    move-result-object p2

    new-instance p3, Ljava/lang/StringBuilder;

    invoke-direct {p3}, Ljava/lang/StringBuilder;-><init>()V

    const-string v0, "\u041c\u0430\u0440\u0448\u0440\u0443\u0442: "

    invoke-virtual {p3, v0}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object p3

    invoke-virtual {p3, p2}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object p2

    invoke-virtual {p2}, Ljava/lang/StringBuilder;->toString()Ljava/lang/String;

    move-result-object p2

    invoke-virtual {p1, p2}, Landroid/widget/Button;->setText(Ljava/lang/CharSequence;)V

    sget-boolean p1, Lorg/garlic/client/GarlicProxyService;->isRunning:Z

    if-eqz p1, :cond_0

    invoke-direct {p0}, Lorg/garlic/client/MainActivity;->stopVpn()V

    iget-object p1, p0, Lorg/garlic/client/MainActivity;->message:Landroid/widget/TextView;

    const-string p2, "\u041c\u0430\u0440\u0448\u0440\u0443\u0442 \u0438\u0437\u043c\u0435\u043d\u0451\u043d. \u041f\u043e\u0434\u043a\u043b\u044e\u0447\u0438\u0442\u0435\u0441\u044c \u0441\u043d\u043e\u0432\u0430"

    invoke-virtual {p1, p2}, Landroid/widget/TextView;->setText(Ljava/lang/CharSequence;)V

    :cond_0
    return-void
.end method

.method synthetic lambda$showLegacy$25$org-garlic-client-MainActivity(I)Z
    .locals 1

    .line 130
    iget-boolean v0, p0, Lorg/garlic/client/MainActivity;->admin:Z

    if-eqz v0, :cond_0

    iget v0, p0, Lorg/garlic/client/MainActivity;->generation:I

    if-ne p1, v0, :cond_0

    invoke-virtual {p0}, Lorg/garlic/client/MainActivity;->isFinishing()Z

    move-result p1

    if-nez p1, :cond_0

    invoke-virtual {p0}, Lorg/garlic/client/MainActivity;->isDestroyed()Z

    move-result p1

    if-nez p1, :cond_0

    const/4 p1, 0x1

    goto :goto_0

    :cond_0
    const/4 p1, 0x0

    :goto_0
    return p1
.end method

.method synthetic lambda$showLogin$2$org-garlic-client-MainActivity(Landroid/widget/Button;Landroid/view/View;)V
    .locals 0

    .line 83
    invoke-direct {p0, p1}, Lorg/garlic/client/MainActivity;->login(Landroid/widget/Button;)V

    return-void
.end method

.method synthetic lambda$showServers$26$org-garlic-client-MainActivity(Landroid/view/View;)V
    .locals 0

    .line 134
    invoke-direct {p0}, Lorg/garlic/client/MainActivity;->showHome()V

    return-void
.end method

.method synthetic lambda$showServers$27$org-garlic-client-MainActivity(Landroid/view/View;)V
    .locals 0

    .line 135
    invoke-direct {p0}, Lorg/garlic/client/MainActivity;->showServers()V

    return-void
.end method

.method synthetic lambda$showServers$28$org-garlic-client-MainActivity(Ljava/lang/String;Lorg/json/JSONObject;IILandroid/view/View;)V
    .locals 0

    .line 146
    const-string p5, "label"

    invoke-virtual {p2, p5}, Lorg/json/JSONObject;->optString(Ljava/lang/String;)Ljava/lang/String;

    move-result-object p2

    invoke-direct {p0, p1, p2, p3, p4}, Lorg/garlic/client/MainActivity;->editServer(Ljava/lang/String;Ljava/lang/String;II)V

    return-void
.end method

.method synthetic lambda$showServers$29$org-garlic-client-MainActivity(ILorg/json/JSONObject;Landroid/widget/LinearLayout;)V
    .locals 18

    .line 137
    move-object/from16 v1, p0

    move-object/from16 v0, p2

    const-string v6, "\n"

    iget v2, v1, Lorg/garlic/client/MainActivity;->generation:I

    move/from16 v3, p1

    if-ne v3, v2, :cond_b

    iget-boolean v2, v1, Lorg/garlic/client/MainActivity;->admin:Z

    if-nez v2, :cond_0

    goto/16 :goto_b

    .line 138
    :cond_0
    :try_start_0
    const-string v2, "ok"

    invoke-virtual {v0, v2}, Lorg/json/JSONObject;->optBoolean(Ljava/lang/String;)Z

    move-result v2

    if-nez v2, :cond_1

    iget-object v2, v1, Lorg/garlic/client/MainActivity;->message:Landroid/widget/TextView;

    const-string v3, "error"

    invoke-virtual {v0, v3}, Lorg/json/JSONObject;->optString(Ljava/lang/String;)Ljava/lang/String;

    move-result-object v0

    invoke-virtual {v2, v0}, Landroid/widget/TextView;->setText(Ljava/lang/CharSequence;)V

    return-void

    .line 139
    :cond_1
    const-string v2, "network"

    invoke-virtual {v0, v2}, Lorg/json/JSONObject;->getJSONObject(Ljava/lang/String;)Lorg/json/JSONObject;

    move-result-object v0

    const-string v2, "observations"

    invoke-virtual {v0, v2}, Lorg/json/JSONObject;->optJSONObject(Ljava/lang/String;)Lorg/json/JSONObject;

    move-result-object v7

    const-string v2, "policies"

    invoke-virtual {v0, v2}, Lorg/json/JSONObject;->optJSONObject(Ljava/lang/String;)Lorg/json/JSONObject;

    move-result-object v8

    const-string v2, "nodes"

    invoke-virtual {v0, v2}, Lorg/json/JSONObject;->getJSONArray(Ljava/lang/String;)Lorg/json/JSONArray;

    move-result-object v9

    iget-object v0, v1, Lorg/garlic/client/MainActivity;->message:Landroid/widget/TextView;

    const-string v2, "\u0421\u043e\u0441\u0442\u043e\u044f\u043d\u0438\u0435 \u043f\u043e \u0434\u0430\u043d\u043d\u044b\u043c \u0441\u0435\u0442\u0438. \u041f\u0440\u044f\u043c\u0430\u044f \u0434\u043e\u0441\u0442\u0443\u043f\u043d\u043e\u0441\u0442\u044c \u043c\u043e\u0436\u0435\u0442 \u043e\u0442\u043b\u0438\u0447\u0430\u0442\u044c\u0441\u044f."

    invoke-virtual {v0, v2}, Landroid/widget/TextView;->setText(Ljava/lang/CharSequence;)V

    .line 140
    const/4 v10, 0x0

    move v11, v10

    :goto_0
    invoke-virtual {v9}, Lorg/json/JSONArray;->length()I

    move-result v0

    if-ge v11, v0, :cond_a

    .line 141
    invoke-virtual {v9, v11}, Lorg/json/JSONArray;->getJSONObject(I)Lorg/json/JSONObject;

    move-result-object v3

    const-string v0, "node_id"

    invoke-virtual {v3, v0}, Lorg/json/JSONObject;->getString(Ljava/lang/String;)Ljava/lang/String;

    move-result-object v2

    const/4 v0, 0x0

    if-nez v7, :cond_2

    move-object v4, v0

    goto :goto_1

    :cond_2
    invoke-virtual {v7, v2}, Lorg/json/JSONObject;->optJSONObject(Ljava/lang/String;)Lorg/json/JSONObject;

    move-result-object v4

    :goto_1
    if-nez v8, :cond_3

    goto :goto_2

    :cond_3
    invoke-virtual {v8, v2}, Lorg/json/JSONObject;->optJSONObject(Ljava/lang/String;)Lorg/json/JSONObject;

    move-result-object v0

    .line 142
    :goto_2
    if-nez v4, :cond_4

    const-wide v12, 0x7fffffffffffffffL

    goto :goto_3

    :cond_4
    invoke-static {}, Ljava/lang/System;->currentTimeMillis()J

    move-result-wide v12

    const-wide/16 v14, 0x3e8

    div-long/2addr v12, v14

    const-string v5, "last_seen"

    invoke-virtual {v4, v5}, Lorg/json/JSONObject;->optLong(Ljava/lang/String;)J

    move-result-wide v14

    sub-long/2addr v12, v14

    const-wide/16 v14, 0x0

    invoke-static {v14, v15, v12, v13}, Ljava/lang/Math;->max(JJ)J

    move-result-wide v12

    .line 143
    :goto_3
    const-string v5, "label"

    invoke-virtual {v3, v5}, Lorg/json/JSONObject;->optString(Ljava/lang/String;)Ljava/lang/String;

    move-result-object v5

    const-string v14, "ru"

    const-string v15, "role"

    invoke-virtual {v3, v15}, Lorg/json/JSONObject;->optString(Ljava/lang/String;)Ljava/lang/String;

    move-result-object v15

    invoke-virtual {v14, v15}, Ljava/lang/String;->equals(Ljava/lang/Object;)Z

    move-result v14

    if-eqz v14, :cond_5

    const-string v14, "\u0420\u0424"

    goto :goto_4

    :cond_5
    const-string v14, "\u0412\u044b\u0445\u043e\u0434"

    :goto_4
    const-string v15, "url"

    invoke-virtual {v3, v15}, Lorg/json/JSONObject;->optString(Ljava/lang/String;)Ljava/lang/String;

    move-result-object v15

    const-wide/16 v16, 0x1e

    cmp-long v12, v12, v16

    if-gez v12, :cond_6

    const-string v12, "\u0412 \u0441\u0435\u0442\u0438"

    goto :goto_5

    :cond_6
    const-string v12, "\u041d\u0435\u0442 \u0441\u0432\u0435\u0436\u0438\u0445 \u0441\u0432\u0435\u0434\u0435\u043d\u0438\u0439"

    :goto_5
    new-instance v13, Ljava/lang/StringBuilder;

    invoke-direct {v13}, Ljava/lang/StringBuilder;-><init>()V

    invoke-virtual {v13, v5}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v5

    const-string v13, " \u00b7 "

    invoke-virtual {v5, v13}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v5

    invoke-virtual {v5, v14}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v5

    invoke-virtual {v5, v6}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v5

    invoke-virtual {v5, v15}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v5

    invoke-virtual {v5, v6}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v5

    invoke-virtual {v5, v12}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v5

    invoke-virtual {v5}, Ljava/lang/StringBuilder;->toString()Ljava/lang/String;

    move-result-object v5

    .line 144
    if-eqz v4, :cond_7

    const-string v12, "active_streams"

    invoke-virtual {v4, v12}, Lorg/json/JSONObject;->optInt(Ljava/lang/String;)I

    move-result v12

    const-string v13, "timeout_count"

    invoke-virtual {v4, v13}, Lorg/json/JSONObject;->optLong(Ljava/lang/String;)J

    move-result-wide v13

    new-instance v4, Ljava/lang/StringBuilder;

    invoke-direct {v4}, Ljava/lang/StringBuilder;-><init>()V

    invoke-virtual {v4, v5}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v4

    const-string v5, " \u00b7 \u043f\u043e\u0442\u043e\u043a\u043e\u0432 "

    invoke-virtual {v4, v5}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v4

    invoke-virtual {v4, v12}, Ljava/lang/StringBuilder;->append(I)Ljava/lang/StringBuilder;

    move-result-object v4

    const-string v5, " \u00b7 \u0442\u0430\u0439\u043c\u0430\u0443\u0442\u043e\u0432 "

    invoke-virtual {v4, v5}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v4

    invoke-virtual {v4, v13, v14}, Ljava/lang/StringBuilder;->append(J)Ljava/lang/StringBuilder;

    move-result-object v4

    invoke-virtual {v4}, Ljava/lang/StringBuilder;->toString()Ljava/lang/String;

    move-result-object v5

    .line 145
    :cond_7
    const/16 v4, 0x80

    if-nez v0, :cond_8

    :goto_6
    goto :goto_7

    :cond_8
    const-string v12, "stream_limit"

    invoke-virtual {v0, v12, v4}, Lorg/json/JSONObject;->optInt(Ljava/lang/String;I)I

    move-result v4

    goto :goto_6

    :goto_7
    const/16 v12, 0x28

    if-nez v0, :cond_9

    :goto_8
    goto :goto_9

    :cond_9
    const-string v13, "worker_limit"

    invoke-virtual {v0, v13, v12}, Lorg/json/JSONObject;->optInt(Ljava/lang/String;I)I

    move-result v12

    goto :goto_8

    .line 146
    :goto_9
    new-instance v0, Ljava/lang/StringBuilder;

    invoke-direct {v0}, Ljava/lang/StringBuilder;-><init>()V

    invoke-virtual {v0, v5}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v0

    const-string v5, "\n\u041b\u0438\u043c\u0438\u0442\u044b: "

    invoke-virtual {v0, v5}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v0

    invoke-virtual {v0, v4}, Ljava/lang/StringBuilder;->append(I)Ljava/lang/StringBuilder;

    move-result-object v0

    const-string v5, " \u043f\u043e\u0442\u043e\u043a\u043e\u0432 / "

    invoke-virtual {v0, v5}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v0

    invoke-virtual {v0, v12}, Ljava/lang/StringBuilder;->append(I)Ljava/lang/StringBuilder;

    move-result-object v0

    const-string v5, " \u043a\u0430\u043d\u0430\u043b\u043e\u0432"

    invoke-virtual {v0, v5}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v0

    invoke-virtual {v0}, Ljava/lang/StringBuilder;->toString()Ljava/lang/String;

    move-result-object v0

    invoke-direct {v1, v0, v10}, Lorg/garlic/client/MainActivity;->button(Ljava/lang/String;Z)Landroid/widget/Button;

    move-result-object v13

    new-instance v0, Landroid/widget/LinearLayout$LayoutParams;

    const/4 v5, -0x1

    const/4 v14, -0x2

    invoke-direct {v0, v5, v14}, Landroid/widget/LinearLayout$LayoutParams;-><init>(II)V

    invoke-virtual {v13, v0}, Landroid/widget/Button;->setLayoutParams(Landroid/view/ViewGroup$LayoutParams;)V

    new-instance v0, Lorg/garlic/client/MainActivity$30;

    move v5, v12

    invoke-direct/range {v0 .. v5}, Lorg/garlic/client/MainActivity$30;-><init>(Lorg/garlic/client/MainActivity;Ljava/lang/String;Lorg/json/JSONObject;II)V

    invoke-virtual {v13, v0}, Landroid/widget/Button;->setOnClickListener(Landroid/view/View$OnClickListener;)V

    move-object/from16 v0, p3

    invoke-virtual {v0, v13}, Landroid/widget/LinearLayout;->addView(Landroid/view/View;)V
    :try_end_0
    .catch Ljava/lang/Exception; {:try_start_0 .. :try_end_0} :catch_0

    .line 140
    add-int/lit8 v11, v11, 0x1

    goto/16 :goto_0

    .line 148
    :cond_a
    goto :goto_a

    :catch_0
    move-exception v0

    iget-object v0, v1, Lorg/garlic/client/MainActivity;->message:Landroid/widget/TextView;

    const-string v1, "\u041d\u0435\u043a\u043e\u0440\u0440\u0435\u043a\u0442\u043d\u044b\u0439 \u043e\u0442\u0432\u0435\u0442 \u0441\u0435\u0442\u0438"

    invoke-virtual {v0, v1}, Landroid/widget/TextView;->setText(Ljava/lang/CharSequence;)V

    :goto_a
    return-void

    .line 137
    :cond_b
    :goto_b
    return-void
.end method

.method synthetic lambda$showServers$30$org-garlic-client-MainActivity()V
    .locals 2

    .line 148
    iget-object v0, p0, Lorg/garlic/client/MainActivity;->message:Landroid/widget/TextView;

    const-string v1, "\u0421\u0435\u0442\u044c \u043d\u0435\u0434\u043e\u0441\u0442\u0443\u043f\u043d\u0430"

    invoke-virtual {v0, v1}, Landroid/widget/TextView;->setText(Ljava/lang/CharSequence;)V

    return-void
.end method

.method synthetic lambda$showServers$31$org-garlic-client-MainActivity(ILandroid/widget/LinearLayout;)V
    .locals 4

    .line 137
    :try_start_0
    new-instance v0, Lorg/json/JSONObject;

    invoke-direct {p0}, Lorg/garlic/client/MainActivity;->dir()Ljava/io/File;

    move-result-object v1

    invoke-virtual {v1}, Ljava/io/File;->getAbsolutePath()Ljava/lang/String;

    move-result-object v1

    invoke-direct {p0}, Lorg/garlic/client/MainActivity;->origins()Ljava/lang/String;

    move-result-object v2

    const-string v3, ""

    invoke-static {v1, v2, v3}, Lorg/garlic/client/GarlicBridge;->adminNetwork(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;

    move-result-object v1

    invoke-direct {v0, v1}, Lorg/json/JSONObject;-><init>(Ljava/lang/String;)V

    new-instance v1, Lorg/garlic/client/MainActivity$28;

    invoke-direct {v1, p0, p1, v0, p2}, Lorg/garlic/client/MainActivity$28;-><init>(Lorg/garlic/client/MainActivity;ILorg/json/JSONObject;Landroid/widget/LinearLayout;)V

    invoke-virtual {p0, v1}, Lorg/garlic/client/MainActivity;->runOnUiThread(Ljava/lang/Runnable;)V
    :try_end_0
    .catchall {:try_start_0 .. :try_end_0} :catchall_0

    .line 148
    goto :goto_0

    :catchall_0
    move-exception p1

    new-instance p1, Lorg/garlic/client/MainActivity$29;

    invoke-direct {p1, p0}, Lorg/garlic/client/MainActivity$29;-><init>(Lorg/garlic/client/MainActivity;)V

    invoke-virtual {p0, p1}, Lorg/garlic/client/MainActivity;->runOnUiThread(Ljava/lang/Runnable;)V

    :goto_0
    return-void
.end method

.method synthetic lambda$showUsers$36$org-garlic-client-MainActivity(Landroid/view/View;)V
    .locals 0

    .line 157
    invoke-direct {p0}, Lorg/garlic/client/MainActivity;->showHome()V

    return-void
.end method

.method synthetic lambda$showUsers$37$org-garlic-client-MainActivity(Landroid/view/View;)V
    .locals 0

    .line 157
    invoke-direct {p0}, Lorg/garlic/client/MainActivity;->editUser()V

    return-void
.end method

.method synthetic lambda$showUsers$38$org-garlic-client-MainActivity(Lorg/json/JSONObject;Landroid/view/View;)V
    .locals 0

    .line 157
    invoke-direct {p0, p1}, Lorg/garlic/client/MainActivity;->userActions(Lorg/json/JSONObject;)V

    return-void
.end method

.method synthetic lambda$showUsers$39$org-garlic-client-MainActivity(ILorg/json/JSONObject;Landroid/widget/LinearLayout;)V
    .locals 9

    .line 157
    const-string v0, "\n"

    const-string v1, ""

    iget v2, p0, Lorg/garlic/client/MainActivity;->generation:I

    if-ne p1, v2, :cond_4

    iget-boolean p1, p0, Lorg/garlic/client/MainActivity;->admin:Z

    if-nez p1, :cond_0

    goto/16 :goto_4

    :cond_0
    :try_start_0
    const-string p1, "ok"

    invoke-virtual {p2, p1}, Lorg/json/JSONObject;->optBoolean(Ljava/lang/String;)Z

    move-result p1

    const/4 v2, 0x0

    if-nez p1, :cond_1

    iput-boolean v2, p0, Lorg/garlic/client/MainActivity;->admin:Z

    invoke-direct {p0}, Lorg/garlic/client/MainActivity;->showHome()V

    iget-object p1, p0, Lorg/garlic/client/MainActivity;->message:Landroid/widget/TextView;

    const-string p2, "\u0414\u043e\u0441\u0442\u0443\u043f \u0430\u0434\u043c\u0438\u043d\u0438\u0441\u0442\u0440\u0430\u0442\u043e\u0440\u0430 \u043d\u0435 \u043f\u043e\u0434\u0442\u0432\u0435\u0440\u0436\u0434\u0451\u043d"

    invoke-virtual {p1, p2}, Landroid/widget/TextView;->setText(Ljava/lang/CharSequence;)V

    return-void

    :cond_1
    iget-object p1, p0, Lorg/garlic/client/MainActivity;->message:Landroid/widget/TextView;

    invoke-virtual {p1, v1}, Landroid/widget/TextView;->setText(Ljava/lang/CharSequence;)V

    const-string p1, "users"

    invoke-virtual {p2, p1}, Lorg/json/JSONObject;->getJSONArray(Ljava/lang/String;)Lorg/json/JSONArray;

    move-result-object p1

    move p2, v2

    :goto_0
    invoke-virtual {p1}, Lorg/json/JSONArray;->length()I

    move-result v3

    if-ge p2, v3, :cond_4

    invoke-virtual {p1, p2}, Lorg/json/JSONArray;->getJSONObject(I)Lorg/json/JSONObject;

    move-result-object v3

    const-string v4, "username"

    invoke-virtual {v3, v4}, Lorg/json/JSONObject;->getString(Ljava/lang/String;)Ljava/lang/String;

    move-result-object v4

    invoke-direct {p0, v3}, Lorg/garlic/client/MainActivity;->userSubscription(Lorg/json/JSONObject;)Ljava/lang/String;

    move-result-object v5

    const-string v6, "enabled"

    invoke-virtual {v3, v6}, Lorg/json/JSONObject;->optBoolean(Ljava/lang/String;)Z

    move-result v6

    if-eqz v6, :cond_3

    move-object v6, v1

    goto :goto_3

    :cond_3
    const-string v6, " \u00b7 \u043e\u0442\u043a\u043b\u044e\u0447\u0451\u043d"

    :goto_3
    const-string v7, "client_usage_display"

    const-string v8, "\u041d\u0435\u0442 \u0441\u0432\u0435\u0434\u0435\u043d\u0438\u0439"

    invoke-virtual {v3, v7, v8}, Lorg/json/JSONObject;->optString(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;

    move-result-object v7

    new-instance v8, Ljava/lang/StringBuilder;

    invoke-direct {v8}, Ljava/lang/StringBuilder;-><init>()V

    invoke-virtual {v8, v4}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v4

    invoke-virtual {v4, v0}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v4

    invoke-virtual {v4, v5}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v4

    invoke-virtual {v4, v6}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v4

    invoke-virtual {v4, v0}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v4

    invoke-virtual {v4, v7}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object v4

    invoke-virtual {v4}, Ljava/lang/StringBuilder;->toString()Ljava/lang/String;

    move-result-object v4

    invoke-direct {p0, v4, v2}, Lorg/garlic/client/MainActivity;->button(Ljava/lang/String;Z)Landroid/widget/Button;

    move-result-object v4

    new-instance v5, Landroid/widget/LinearLayout$LayoutParams;

    const/4 v6, -0x1

    const/4 v7, -0x2

    invoke-direct {v5, v6, v7}, Landroid/widget/LinearLayout$LayoutParams;-><init>(II)V

    invoke-virtual {v4, v5}, Landroid/widget/Button;->setLayoutParams(Landroid/view/ViewGroup$LayoutParams;)V

    const/16 v5, 0x10

    invoke-direct {p0, v5}, Lorg/garlic/client/MainActivity;->dp(I)I

    move-result v6

    const/16 v7, 0xe

    invoke-direct {p0, v7}, Lorg/garlic/client/MainActivity;->dp(I)I

    move-result v8

    invoke-direct {p0, v5}, Lorg/garlic/client/MainActivity;->dp(I)I

    move-result v5

    invoke-direct {p0, v7}, Lorg/garlic/client/MainActivity;->dp(I)I

    move-result v7

    invoke-virtual {v4, v6, v8, v5, v7}, Landroid/widget/Button;->setPadding(IIII)V

    new-instance v5, Lorg/garlic/client/MainActivity$15;

    invoke-direct {v5, p0, v3}, Lorg/garlic/client/MainActivity$15;-><init>(Lorg/garlic/client/MainActivity;Lorg/json/JSONObject;)V

    invoke-virtual {v4, v5}, Landroid/widget/Button;->setOnClickListener(Landroid/view/View$OnClickListener;)V

    invoke-virtual {p3, v4}, Landroid/widget/LinearLayout;->addView(Landroid/view/View;)V
    :try_end_0
    .catch Ljava/lang/Exception; {:try_start_0 .. :try_end_0} :catch_0

    add-int/lit8 p2, p2, 0x1

    goto/16 :goto_0

    :catch_0
    move-exception p1

    iget-object p1, p0, Lorg/garlic/client/MainActivity;->message:Landroid/widget/TextView;

    const-string p2, "\u041e\u0448\u0438\u0431\u043a\u0430 \u0441\u043f\u0438\u0441\u043a\u0430 \u043f\u043e\u043b\u044c\u0437\u043e\u0432\u0430\u0442\u0435\u043b\u0435\u0439"

    invoke-virtual {p1, p2}, Landroid/widget/TextView;->setText(Ljava/lang/CharSequence;)V

    :cond_4
    :goto_4
    return-void
.end method

.method synthetic lambda$showUsers$40$org-garlic-client-MainActivity()V
    .locals 2

    .line 157
    iget-object v0, p0, Lorg/garlic/client/MainActivity;->message:Landroid/widget/TextView;

    const-string v1, "\u0421\u0435\u0440\u0432\u0435\u0440 \u043d\u0435\u0434\u043e\u0441\u0442\u0443\u043f\u0435\u043d"

    invoke-virtual {v0, v1}, Landroid/widget/TextView;->setText(Ljava/lang/CharSequence;)V

    return-void
.end method

.method synthetic lambda$showUsers$41$org-garlic-client-MainActivity(ILandroid/widget/LinearLayout;)V
    .locals 3

    .line 157
    :try_start_0
    new-instance v0, Lorg/json/JSONObject;

    invoke-direct {p0}, Lorg/garlic/client/MainActivity;->dir()Ljava/io/File;

    move-result-object v1

    invoke-virtual {v1}, Ljava/io/File;->getAbsolutePath()Ljava/lang/String;

    move-result-object v1

    invoke-direct {p0}, Lorg/garlic/client/MainActivity;->origins()Ljava/lang/String;

    move-result-object v2

    invoke-static {v1, v2}, Lorg/garlic/client/GarlicBridge;->adminListUsers(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;

    move-result-object v1

    invoke-direct {v0, v1}, Lorg/json/JSONObject;-><init>(Ljava/lang/String;)V

    new-instance v1, Lorg/garlic/client/MainActivity$42;

    invoke-direct {v1, p0, p1, v0, p2}, Lorg/garlic/client/MainActivity$42;-><init>(Lorg/garlic/client/MainActivity;ILorg/json/JSONObject;Landroid/widget/LinearLayout;)V

    invoke-virtual {p0, v1}, Lorg/garlic/client/MainActivity;->runOnUiThread(Ljava/lang/Runnable;)V
    :try_end_0
    .catchall {:try_start_0 .. :try_end_0} :catchall_0

    goto :goto_0

    :catchall_0
    move-exception p1

    new-instance p1, Lorg/garlic/client/MainActivity$43;

    invoke-direct {p1, p0}, Lorg/garlic/client/MainActivity$43;-><init>(Lorg/garlic/client/MainActivity;)V

    invoke-virtual {p0, p1}, Lorg/garlic/client/MainActivity;->runOnUiThread(Ljava/lang/Runnable;)V

    :goto_0
    return-void
.end method

.method synthetic lambda$subscriptionDialog$43$org-garlic-client-MainActivity(Ljava/util/Calendar;Landroid/widget/Button;Landroid/view/View;)V
    .locals 6

    .line 165
    new-instance v0, Landroid/app/DatePickerDialog;

    new-instance v2, Lorg/garlic/client/MainActivity$53;

    invoke-direct {v2, p1, p2}, Lorg/garlic/client/MainActivity$53;-><init>(Ljava/util/Calendar;Landroid/widget/Button;)V

    const/4 p2, 0x1

    invoke-virtual {p1, p2}, Ljava/util/Calendar;->get(I)I

    move-result v3

    const/4 p2, 0x2

    invoke-virtual {p1, p2}, Ljava/util/Calendar;->get(I)I

    move-result v4

    const/4 p2, 0x5

    invoke-virtual {p1, p2}, Ljava/util/Calendar;->get(I)I

    move-result v5

    move-object v1, p0

    invoke-direct/range {v0 .. v5}, Landroid/app/DatePickerDialog;-><init>(Landroid/content/Context;Landroid/app/DatePickerDialog$OnDateSetListener;III)V

    invoke-virtual {v0}, Landroid/app/DatePickerDialog;->show()V

    return-void
.end method

.method synthetic lambda$subscriptionDialog$45$org-garlic-client-MainActivity(Ljava/util/Calendar;Ljava/lang/String;Landroid/widget/EditText;Landroid/widget/CheckBox;Landroid/content/DialogInterface;I)V
    .locals 4

    .line 168
    const/16 p5, 0xb

    const/16 p6, 0x17

    :try_start_0
    invoke-virtual {p1, p5, p6}, Ljava/util/Calendar;->set(II)V

    const/16 p5, 0xc

    const/16 p6, 0x3b

    invoke-virtual {p1, p5, p6}, Ljava/util/Calendar;->set(II)V

    const/16 p5, 0xd

    invoke-virtual {p1, p5, p6}, Ljava/util/Calendar;->set(II)V
    :try_end_0
    .catch Ljava/lang/Exception; {:try_start_0 .. :try_end_0} :catch_0

    const-string p5, "subscription"

    if-nez p2, :cond_0

    :try_start_1
    const-string p6, "create"

    goto :goto_0

    :cond_0
    move-object p6, p5

    :goto_0
    if-nez p2, :cond_1

    invoke-virtual {p3}, Landroid/widget/EditText;->getText()Landroid/text/Editable;

    move-result-object p3

    invoke-virtual {p3}, Ljava/lang/Object;->toString()Ljava/lang/String;

    move-result-object p3

    invoke-virtual {p3}, Ljava/lang/String;->trim()Ljava/lang/String;

    move-result-object p3

    goto :goto_1

    :cond_1
    move-object p3, p2

    :goto_1
    invoke-direct {p0, p6, p3}, Lorg/garlic/client/MainActivity;->action(Ljava/lang/String;Ljava/lang/String;)Lorg/json/JSONObject;

    move-result-object p3

    invoke-virtual {p4}, Landroid/widget/CheckBox;->isChecked()Z

    move-result p4

    if-eqz p4, :cond_2

    const-string p1, "unlimited"

    goto :goto_2

    :cond_2
    invoke-virtual {p1}, Ljava/util/Calendar;->getTimeInMillis()J

    move-result-wide v0

    const-wide/16 v2, 0x3e8

    div-long/2addr v0, v2

    invoke-static {v0, v1}, Ljava/lang/String;->valueOf(J)Ljava/lang/String;

    move-result-object p1

    :goto_2
    invoke-virtual {p3, p5, p1}, Lorg/json/JSONObject;->put(Ljava/lang/String;Ljava/lang/Object;)Lorg/json/JSONObject;

    if-nez p2, :cond_3

    const-string p1, "role"

    const-string p2, "client"

    invoke-virtual {p3, p1, p2}, Lorg/json/JSONObject;->put(Ljava/lang/String;Ljava/lang/Object;)Lorg/json/JSONObject;

    :cond_3
    invoke-direct {p0, p3}, Lorg/garlic/client/MainActivity;->manage(Lorg/json/JSONObject;)V
    :try_end_1
    .catch Ljava/lang/Exception; {:try_start_1 .. :try_end_1} :catch_0

    goto :goto_3

    :catch_0
    move-exception p1

    .line 169
    :goto_3
    return-void
.end method

.method synthetic lambda$takeover$18$org-garlic-client-MainActivity(ILorg/json/JSONObject;)V
    .locals 2

    .line 116
    iget v0, p0, Lorg/garlic/client/MainActivity;->generation:I

    if-ne p1, v0, :cond_2

    iget-boolean p1, p0, Lorg/garlic/client/MainActivity;->loggedIn:Z

    if-nez p1, :cond_0

    goto :goto_0

    :cond_0
    const-string p1, "ok"

    invoke-virtual {p2, p1}, Lorg/json/JSONObject;->optBoolean(Ljava/lang/String;)Z

    move-result p1

    iget-object v0, p0, Lorg/garlic/client/MainActivity;->message:Landroid/widget/TextView;

    if-eqz p1, :cond_1

    const-string p1, "\u0414\u043e\u0441\u0442\u0443\u043f \u043f\u0435\u0440\u0435\u043a\u043b\u044e\u0447\u0451\u043d"

    invoke-virtual {v0, p1}, Landroid/widget/TextView;->setText(Ljava/lang/CharSequence;)V

    invoke-direct {p0}, Lorg/garlic/client/MainActivity;->startVpn()V

    goto :goto_0

    :cond_1
    const-string p1, "error"

    const-string v1, "\u041d\u0435 \u0443\u0434\u0430\u043b\u043e\u0441\u044c \u043f\u0435\u0440\u0435\u043a\u043b\u044e\u0447\u0438\u0442\u044c"

    invoke-virtual {p2, p1, v1}, Lorg/json/JSONObject;->optString(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;

    move-result-object p1

    invoke-virtual {v0, p1}, Landroid/widget/TextView;->setText(Ljava/lang/CharSequence;)V

    :cond_2
    :goto_0
    return-void
.end method

.method synthetic lambda$takeover$19$org-garlic-client-MainActivity(I)V
    .locals 1

    .line 116
    iget v0, p0, Lorg/garlic/client/MainActivity;->generation:I

    if-ne p1, v0, :cond_0

    iget-boolean p1, p0, Lorg/garlic/client/MainActivity;->loggedIn:Z

    if-eqz p1, :cond_0

    iget-object p1, p0, Lorg/garlic/client/MainActivity;->message:Landroid/widget/TextView;

    const-string v0, "\u041d\u0435 \u0443\u0434\u0430\u043b\u043e\u0441\u044c \u0441\u0432\u044f\u0437\u0430\u0442\u044c\u0441\u044f \u0441 \u0441\u0435\u0440\u0432\u0435\u0440\u043e\u043c"

    invoke-virtual {p1, v0}, Landroid/widget/TextView;->setText(Ljava/lang/CharSequence;)V

    :cond_0
    return-void
.end method

.method synthetic lambda$takeover$20$org-garlic-client-MainActivity(I)V
    .locals 3

    .line 116
    :try_start_0
    new-instance v0, Lorg/json/JSONObject;

    invoke-direct {p0}, Lorg/garlic/client/MainActivity;->dir()Ljava/io/File;

    move-result-object v1

    invoke-virtual {v1}, Ljava/io/File;->getAbsolutePath()Ljava/lang/String;

    move-result-object v1

    invoke-direct {p0}, Lorg/garlic/client/MainActivity;->origins()Ljava/lang/String;

    move-result-object v2

    invoke-static {v1, v2}, Lorg/garlic/client/GarlicBridge;->takeover(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;

    move-result-object v1

    invoke-direct {v0, v1}, Lorg/json/JSONObject;-><init>(Ljava/lang/String;)V

    new-instance v1, Lorg/garlic/client/MainActivity$36;

    invoke-direct {v1, p0, p1, v0}, Lorg/garlic/client/MainActivity$36;-><init>(Lorg/garlic/client/MainActivity;ILorg/json/JSONObject;)V

    invoke-virtual {p0, v1}, Lorg/garlic/client/MainActivity;->runOnUiThread(Ljava/lang/Runnable;)V
    :try_end_0
    .catchall {:try_start_0 .. :try_end_0} :catchall_0

    goto :goto_0

    :catchall_0
    move-exception v0

    new-instance v0, Lorg/garlic/client/MainActivity$37;

    invoke-direct {v0, p0, p1}, Lorg/garlic/client/MainActivity$37;-><init>(Lorg/garlic/client/MainActivity;I)V

    invoke-virtual {p0, v0}, Lorg/garlic/client/MainActivity;->runOnUiThread(Ljava/lang/Runnable;)V

    :goto_0
    return-void
.end method

.method synthetic lambda$userActions$46$org-garlic-client-MainActivity(Ljava/lang/String;Landroid/content/DialogInterface;I)V
    .locals 0

    .line 176
    const-string p2, "role"

    :try_start_0
    invoke-direct {p0, p2, p1}, Lorg/garlic/client/MainActivity;->action(Ljava/lang/String;Ljava/lang/String;)Lorg/json/JSONObject;

    move-result-object p1

    if-nez p3, :cond_0

    const-string p3, "client"

    goto :goto_0

    :cond_0
    const-string p3, "admin"

    :goto_0
    invoke-virtual {p1, p2, p3}, Lorg/json/JSONObject;->put(Ljava/lang/String;Ljava/lang/Object;)Lorg/json/JSONObject;

    invoke-direct {p0, p1}, Lorg/garlic/client/MainActivity;->manage(Lorg/json/JSONObject;)V
    :try_end_0
    .catch Ljava/lang/Exception; {:try_start_0 .. :try_end_0} :catch_0

    goto :goto_1

    :catch_0
    move-exception p1

    :goto_1
    return-void
.end method

.method synthetic lambda$userActions$47$org-garlic-client-MainActivity(Ljava/lang/String;Ljava/lang/String;Landroid/content/DialogInterface;I)V
    .locals 0

    .line 178
    invoke-direct {p0, p1, p2}, Lorg/garlic/client/MainActivity;->action(Ljava/lang/String;Ljava/lang/String;)Lorg/json/JSONObject;

    move-result-object p1

    invoke-direct {p0, p1}, Lorg/garlic/client/MainActivity;->manage(Lorg/json/JSONObject;)V

    return-void
.end method

.method synthetic lambda$userActions$48$org-garlic-client-MainActivity(Lorg/json/JSONObject;Ljava/lang/String;ZLandroid/content/DialogInterface;I)V
    .locals 0

    .line 175
    const/4 p4, 0x1

    if-ne p5, p4, :cond_0

    invoke-direct {p0, p1}, Lorg/garlic/client/MainActivity;->subscriptionDialog(Lorg/json/JSONObject;)V

    return-void

    .line 176
    :cond_0
    const/4 p1, 0x3

    if-ne p5, p1, :cond_1

    new-instance p1, Landroid/app/AlertDialog$Builder;

    invoke-direct {p1, p0}, Landroid/app/AlertDialog$Builder;-><init>(Landroid/content/Context;)V

    const-string p3, "\u0420\u043e\u043b\u044c"

    invoke-virtual {p1, p3}, Landroid/app/AlertDialog$Builder;->setTitle(Ljava/lang/CharSequence;)Landroid/app/AlertDialog$Builder;

    move-result-object p1

    const-string p3, "\u041a\u043b\u0438\u0435\u043d\u0442"

    const-string p4, "\u0410\u0434\u043c\u0438\u043d\u0438\u0441\u0442\u0440\u0430\u0442\u043e\u0440"

    filled-new-array {p3, p4}, [Ljava/lang/String;

    move-result-object p3

    new-instance p4, Lorg/garlic/client/MainActivity$10;

    invoke-direct {p4, p0, p2}, Lorg/garlic/client/MainActivity$10;-><init>(Lorg/garlic/client/MainActivity;Ljava/lang/String;)V

    invoke-virtual {p1, p3, p4}, Landroid/app/AlertDialog$Builder;->setItems([Ljava/lang/CharSequence;Landroid/content/DialogInterface$OnClickListener;)Landroid/app/AlertDialog$Builder;

    move-result-object p1

    invoke-virtual {p1}, Landroid/app/AlertDialog$Builder;->show()Landroid/app/AlertDialog;

    return-void

    .line 177
    :cond_1
    if-nez p5, :cond_2

    const-string p1, "issue_key"

    goto :goto_0

    :cond_2
    const/4 p1, 0x2

    if-ne p5, p1, :cond_4

    if-eqz p3, :cond_3

    const-string p1, "disable"

    goto :goto_0

    :cond_3
    const-string p1, "enable"

    goto :goto_0

    :cond_4
    const-string p1, "remove"

    .line 178
    :goto_0
    const/4 p3, 0x4

    if-ne p5, p3, :cond_5

    new-instance p3, Landroid/app/AlertDialog$Builder;

    invoke-direct {p3, p0}, Landroid/app/AlertDialog$Builder;-><init>(Landroid/content/Context;)V

    new-instance p4, Ljava/lang/StringBuilder;

    invoke-direct {p4}, Ljava/lang/StringBuilder;-><init>()V

    const-string p5, "\u0423\u0434\u0430\u043b\u0438\u0442\u044c "

    invoke-virtual {p4, p5}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object p4

    invoke-virtual {p4, p2}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object p4

    const-string p5, "?"

    invoke-virtual {p4, p5}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    move-result-object p4

    invoke-virtual {p4}, Ljava/lang/StringBuilder;->toString()Ljava/lang/String;

    move-result-object p4

    invoke-virtual {p3, p4}, Landroid/app/AlertDialog$Builder;->setTitle(Ljava/lang/CharSequence;)Landroid/app/AlertDialog$Builder;

    move-result-object p3

    const-string p4, "\u041e\u0442\u043c\u0435\u043d\u0430"

    const/4 p5, 0x0

    invoke-virtual {p3, p4, p5}, Landroid/app/AlertDialog$Builder;->setNegativeButton(Ljava/lang/CharSequence;Landroid/content/DialogInterface$OnClickListener;)Landroid/app/AlertDialog$Builder;

    move-result-object p3

    new-instance p4, Lorg/garlic/client/MainActivity$11;

    invoke-direct {p4, p0, p1, p2}, Lorg/garlic/client/MainActivity$11;-><init>(Lorg/garlic/client/MainActivity;Ljava/lang/String;Ljava/lang/String;)V

    const-string p1, "\u0423\u0434\u0430\u043b\u0438\u0442\u044c"

    invoke-virtual {p3, p1, p4}, Landroid/app/AlertDialog$Builder;->setPositiveButton(Ljava/lang/CharSequence;Landroid/content/DialogInterface$OnClickListener;)Landroid/app/AlertDialog$Builder;

    move-result-object p1

    invoke-virtual {p1}, Landroid/app/AlertDialog$Builder;->show()Landroid/app/AlertDialog;

    goto :goto_1

    :cond_5
    invoke-direct {p0, p1, p2}, Lorg/garlic/client/MainActivity;->action(Ljava/lang/String;Ljava/lang/String;)Lorg/json/JSONObject;

    move-result-object p1

    invoke-direct {p0, p1}, Lorg/garlic/client/MainActivity;->manage(Lorg/json/JSONObject;)V

    .line 179
    :goto_1
    return-void
.end method

.method protected onActivityResult(IILandroid/content/Intent;)V
    .locals 1

    .line 110
    invoke-super {p0, p1, p2, p3}, Landroid/app/Activity;->onActivityResult(IILandroid/content/Intent;)V
    invoke-static {p0, p1, p2, p3}, Lorg/garlic/client/BackgroundSettings;->exportResult(Landroid/app/Activity;IILandroid/content/Intent;)V

    const/16 p3, 0x64

    if-ne p1, p3, :cond_0

    const/4 p1, -0x1

    if-ne p2, p1, :cond_0

    iget-boolean p1, p0, Lorg/garlic/client/MainActivity;->loggedIn:Z

    if-eqz p1, :cond_0

    iget p1, p0, Lorg/garlic/client/MainActivity;->vpnPermissionGeneration:I

    iget p2, p0, Lorg/garlic/client/MainActivity;->generation:I

    if-ne p1, p2, :cond_0

    new-instance p1, Landroid/content/Intent;

    const-class p2, Lorg/garlic/client/GarlicProxyService;

    invoke-direct {p1, p0, p2}, Landroid/content/Intent;-><init>(Landroid/content/Context;Ljava/lang/Class;)V

    iget-object p2, p0, Lorg/garlic/client/MainActivity;->prefs:Landroid/content/SharedPreferences;

    const-string p3, "auto"

    const-string v0, "mode"

    invoke-interface {p2, v0, p3}, Landroid/content/SharedPreferences;->getString(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;

    move-result-object p2

    invoke-virtual {p1, v0, p2}, Landroid/content/Intent;->putExtra(Ljava/lang/String;Ljava/lang/String;)Landroid/content/Intent;

    iget-object p2, p0, Lorg/garlic/client/MainActivity;->prefs:Landroid/content/SharedPreferences;

    const-string p3, "direct_urls"

    const-string v0, ""

    invoke-interface {p2, p3, v0}, Landroid/content/SharedPreferences;->getString(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;

    move-result-object p2

    invoke-virtual {p1, p3, p2}, Landroid/content/Intent;->putExtra(Ljava/lang/String;Ljava/lang/String;)Landroid/content/Intent;

    iget-object p2, p0, Lorg/garlic/client/MainActivity;->prefs:Landroid/content/SharedPreferences;

    const-string p3, "ru_urls"

    invoke-interface {p2, p3, v0}, Landroid/content/SharedPreferences;->getString(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;

    move-result-object p2

    invoke-virtual {p1, p3, p2}, Landroid/content/Intent;->putExtra(Ljava/lang/String;Ljava/lang/String;)Landroid/content/Intent;

    const/4 p2, 0x1

    iput-boolean p2, p0, Lorg/garlic/client/MainActivity;->starting:Z

    iget-object p2, p0, Lorg/garlic/client/MainActivity;->message:Landroid/widget/TextView;

    invoke-virtual {p2, v0}, Landroid/widget/TextView;->setText(Ljava/lang/CharSequence;)V

    invoke-virtual {p0, p1}, Lorg/garlic/client/MainActivity;->startForegroundService(Landroid/content/Intent;)Landroid/content/ComponentName;

    :cond_0
    return-void
.end method

.method public onCreate(Landroid/os/Bundle;)V
    .locals 5

    .line 74
    invoke-super {p0, p1}, Landroid/app/Activity;->onCreate(Landroid/os/Bundle;)V
    invoke-static {p0}, Lorg/garlic/client/BackgroundSettings;->recordLaunch(Landroid/app/Activity;)V

    const/4 p1, 0x1

    invoke-virtual {p0, p1}, Lorg/garlic/client/MainActivity;->requestWindowFeature(I)Z

    invoke-virtual {p0}, Lorg/garlic/client/MainActivity;->getWindow()Landroid/view/Window;

    move-result-object v0

    iget v1, p0, Lorg/garlic/client/MainActivity;->bg:I

    invoke-virtual {v0, v1}, Landroid/view/Window;->setStatusBarColor(I)V

    invoke-virtual {p0}, Lorg/garlic/client/MainActivity;->getWindow()Landroid/view/Window;

    move-result-object v0

    iget v1, p0, Lorg/garlic/client/MainActivity;->bg:I

    invoke-virtual {v0, v1}, Landroid/view/Window;->setNavigationBarColor(I)V

    const-string v0, "garlic_prefs"

    const/4 v1, 0x0

    invoke-virtual {p0, v0, v1}, Lorg/garlic/client/MainActivity;->getSharedPreferences(Ljava/lang/String;I)Landroid/content/SharedPreferences;

    move-result-object v0

    iput-object v0, p0, Lorg/garlic/client/MainActivity;->prefs:Landroid/content/SharedPreferences;

    .line 75
    sget v0, Landroid/os/Build$VERSION;->SDK_INT:I

    const/16 v2, 0x21

    if-lt v0, v2, :cond_0

    const-string v0, "android.permission.POST_NOTIFICATIONS"

    invoke-virtual {p0, v0}, Lorg/garlic/client/MainActivity;->checkSelfPermission(Ljava/lang/String;)I

    move-result v2

    if-eqz v2, :cond_0

    filled-new-array {v0}, [Ljava/lang/String;

    move-result-object v0

    const/16 v2, 0xc8

    invoke-virtual {p0, v0, v2}, Lorg/garlic/client/MainActivity;->requestPermissions([Ljava/lang/String;I)V

    .line 76
    :cond_0
    new-instance v0, Ljava/io/File;

    invoke-direct {p0}, Lorg/garlic/client/MainActivity;->dir()Ljava/io/File;

    move-result-object v2

    const-string v3, "client.sessions.json"

    invoke-direct {v0, v2, v3}, Ljava/io/File;-><init>(Ljava/io/File;Ljava/lang/String;)V

    invoke-virtual {v0}, Ljava/io/File;->isFile()Z

    move-result v0

    if-eqz v0, :cond_2

    .line 77
    :try_start_0
    new-instance v0, Lorg/json/JSONObject;

    iget-object v2, p0, Lorg/garlic/client/MainActivity;->prefs:Landroid/content/SharedPreferences;

    const-string v3, "account_cache"

    const-string v4, "{}"

    invoke-interface {v2, v3, v4}, Landroid/content/SharedPreferences;->getString(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;

    move-result-object v2

    invoke-direct {v0, v2}, Lorg/json/JSONObject;-><init>(Ljava/lang/String;)V

    iput-object v0, p0, Lorg/garlic/client/MainActivity;->account:Lorg/json/JSONObject;
    :try_end_0
    .catch Ljava/lang/Exception; {:try_start_0 .. :try_end_0} :catch_0

    goto :goto_0

    :catch_0
    move-exception v0

    new-instance v0, Lorg/json/JSONObject;

    invoke-direct {v0}, Lorg/json/JSONObject;-><init>()V

    iput-object v0, p0, Lorg/garlic/client/MainActivity;->account:Lorg/json/JSONObject;

    .line 78
    :goto_0
    iget-object v0, p0, Lorg/garlic/client/MainActivity;->account:Lorg/json/JSONObject;

    const-string v2, "principal"

    invoke-virtual {v0, v2}, Lorg/json/JSONObject;->optJSONObject(Ljava/lang/String;)Lorg/json/JSONObject;

    move-result-object v0

    if-eqz v0, :cond_1

    move v1, p1

    :cond_1
    iput-boolean v1, p0, Lorg/garlic/client/MainActivity;->roleVerified:Z

    iput-boolean p1, p0, Lorg/garlic/client/MainActivity;->loggedIn:Z

    invoke-direct {p0}, Lorg/garlic/client/MainActivity;->showHome()V

    goto :goto_1

    .line 79
    :cond_2
    invoke-direct {p0}, Lorg/garlic/client/MainActivity;->showLogin()V

    .line 80
    :goto_1
    invoke-direct {p0}, Lorg/garlic/client/MainActivity;->refreshAccount()V

    iget-object p1, p0, Lorg/garlic/client/MainActivity;->handler:Landroid/os/Handler;

    iget-object v0, p0, Lorg/garlic/client/MainActivity;->statusPoll:Ljava/lang/Runnable;

    invoke-virtual {p1, v0}, Landroid/os/Handler;->post(Ljava/lang/Runnable;)Z

    .line 81
    return-void
.end method

.method protected onDestroy()V
    .locals 2

    .line 183
    iget v0, p0, Lorg/garlic/client/MainActivity;->generation:I

    add-int/lit8 v0, v0, 0x1

    iput v0, p0, Lorg/garlic/client/MainActivity;->generation:I

    iget-object v0, p0, Lorg/garlic/client/MainActivity;->handler:Landroid/os/Handler;

    iget-object v1, p0, Lorg/garlic/client/MainActivity;->statusPoll:Ljava/lang/Runnable;

    invoke-virtual {v0, v1}, Landroid/os/Handler;->removeCallbacks(Ljava/lang/Runnable;)V

    iget-object v0, p0, Lorg/garlic/client/MainActivity;->worker:Ljava/util/concurrent/ExecutorService;

    invoke-interface {v0}, Ljava/util/concurrent/ExecutorService;->shutdown()V

    invoke-super {p0}, Landroid/app/Activity;->onDestroy()V

    return-void
.end method

.method protected onStart()V
    .locals 0

    .line 52
    invoke-super {p0}, Landroid/app/Activity;->onStart()V

    invoke-static {p0}, Lorg/garlic/client/CoreLoader;->check(Landroid/content/Context;)V

    return-void
.end method
