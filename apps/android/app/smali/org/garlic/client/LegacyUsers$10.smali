.class public final synthetic Lorg/garlic/client/LegacyUsers$10;
.super Ljava/lang/Object;
.source "D8$$SyntheticClass"

# interfaces
.implements Landroid/content/DialogInterface$OnClickListener;


# annotations
.annotation runtime Lcom/android/tools/r8/annotations/LambdaMethod;
    holder = "Lorg/garlic/client/LegacyUsers;"
    method = "lambda$create$11"
    proto = "(Ljava/util/Map;Landroid/widget/EditText;Ljava/util/Calendar;Landroid/widget/CheckBox;Landroid/content/DialogInterface;I)V"
.end annotation


# instance fields
.field public final synthetic f$0:Lorg/garlic/client/LegacyUsers;

.field public final synthetic f$1:Ljava/util/Map;

.field public final synthetic f$2:Landroid/widget/EditText;

.field public final synthetic f$3:Ljava/util/Calendar;

.field public final synthetic f$4:Landroid/widget/CheckBox;


# direct methods
.method public synthetic constructor <init>(Lorg/garlic/client/LegacyUsers;Ljava/util/Map;Landroid/widget/EditText;Ljava/util/Calendar;Landroid/widget/CheckBox;)V
    .locals 0

    .line 0
    invoke-direct {p0}, Ljava/lang/Object;-><init>()V

    iput-object p1, p0, Lorg/garlic/client/LegacyUsers$10;->f$0:Lorg/garlic/client/LegacyUsers;

    iput-object p2, p0, Lorg/garlic/client/LegacyUsers$10;->f$1:Ljava/util/Map;

    iput-object p3, p0, Lorg/garlic/client/LegacyUsers$10;->f$2:Landroid/widget/EditText;

    iput-object p4, p0, Lorg/garlic/client/LegacyUsers$10;->f$3:Ljava/util/Calendar;

    iput-object p5, p0, Lorg/garlic/client/LegacyUsers$10;->f$4:Landroid/widget/CheckBox;

    return-void
.end method


# virtual methods
.method public final onClick(Landroid/content/DialogInterface;I)V
    .locals 7

    .line 0
    iget-object v0, p0, Lorg/garlic/client/LegacyUsers$10;->f$0:Lorg/garlic/client/LegacyUsers;

    iget-object v1, p0, Lorg/garlic/client/LegacyUsers$10;->f$1:Ljava/util/Map;

    iget-object v2, p0, Lorg/garlic/client/LegacyUsers$10;->f$2:Landroid/widget/EditText;

    iget-object v3, p0, Lorg/garlic/client/LegacyUsers$10;->f$3:Ljava/util/Calendar;

    iget-object v4, p0, Lorg/garlic/client/LegacyUsers$10;->f$4:Landroid/widget/CheckBox;

    move-object v5, p1

    move v6, p2

    invoke-virtual/range {v0 .. v6}, Lorg/garlic/client/LegacyUsers;->lambda$create$11$org-garlic-client-LegacyUsers(Ljava/util/Map;Landroid/widget/EditText;Ljava/util/Calendar;Landroid/widget/CheckBox;Landroid/content/DialogInterface;I)V

    return-void
.end method
