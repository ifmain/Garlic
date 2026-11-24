.class public final synthetic Lorg/garlic/client/LegacyUsers$11;
.super Ljava/lang/Object;
.source "D8$$SyntheticClass"

# interfaces
.implements Landroid/content/DialogInterface$OnClickListener;


# annotations
.annotation runtime Lcom/android/tools/r8/annotations/LambdaMethod;
    holder = "Lorg/garlic/client/LegacyUsers;"
    method = "lambda$options$6"
    proto = "(Ljava/lang/String;Landroid/widget/EditText;Landroid/content/DialogInterface;I)V"
.end annotation


# instance fields
.field public final synthetic f$0:Lorg/garlic/client/LegacyUsers;

.field public final synthetic f$1:Ljava/lang/String;

.field public final synthetic f$2:Landroid/widget/EditText;


# direct methods
.method public synthetic constructor <init>(Lorg/garlic/client/LegacyUsers;Ljava/lang/String;Landroid/widget/EditText;)V
    .locals 0

    .line 0
    invoke-direct {p0}, Ljava/lang/Object;-><init>()V

    iput-object p1, p0, Lorg/garlic/client/LegacyUsers$11;->f$0:Lorg/garlic/client/LegacyUsers;

    iput-object p2, p0, Lorg/garlic/client/LegacyUsers$11;->f$1:Ljava/lang/String;

    iput-object p3, p0, Lorg/garlic/client/LegacyUsers$11;->f$2:Landroid/widget/EditText;

    return-void
.end method


# virtual methods
.method public final onClick(Landroid/content/DialogInterface;I)V
    .locals 3

    .line 0
    iget-object v0, p0, Lorg/garlic/client/LegacyUsers$11;->f$0:Lorg/garlic/client/LegacyUsers;

    iget-object v1, p0, Lorg/garlic/client/LegacyUsers$11;->f$1:Ljava/lang/String;

    iget-object v2, p0, Lorg/garlic/client/LegacyUsers$11;->f$2:Landroid/widget/EditText;

    invoke-virtual {v0, v1, v2, p1, p2}, Lorg/garlic/client/LegacyUsers;->lambda$options$6$org-garlic-client-LegacyUsers(Ljava/lang/String;Landroid/widget/EditText;Landroid/content/DialogInterface;I)V

    return-void
.end method
