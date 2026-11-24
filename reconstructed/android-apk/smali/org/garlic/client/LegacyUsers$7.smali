.class public final synthetic Lorg/garlic/client/LegacyUsers$7;
.super Ljava/lang/Object;
.source "D8$$SyntheticClass"

# interfaces
.implements Landroid/content/DialogInterface$OnClickListener;


# annotations
.annotation runtime Lcom/android/tools/r8/annotations/LambdaMethod;
    holder = "Lorg/garlic/client/LegacyUsers;"
    method = "lambda$options$7"
    proto = "(Ljava/lang/String;Landroid/content/DialogInterface;I)V"
.end annotation


# instance fields
.field public final synthetic f$0:Lorg/garlic/client/LegacyUsers;

.field public final synthetic f$1:Ljava/lang/String;


# direct methods
.method public synthetic constructor <init>(Lorg/garlic/client/LegacyUsers;Ljava/lang/String;)V
    .locals 0

    .line 0
    invoke-direct {p0}, Ljava/lang/Object;-><init>()V

    iput-object p1, p0, Lorg/garlic/client/LegacyUsers$7;->f$0:Lorg/garlic/client/LegacyUsers;

    iput-object p2, p0, Lorg/garlic/client/LegacyUsers$7;->f$1:Ljava/lang/String;

    return-void
.end method


# virtual methods
.method public final onClick(Landroid/content/DialogInterface;I)V
    .locals 2

    .line 0
    iget-object v0, p0, Lorg/garlic/client/LegacyUsers$7;->f$0:Lorg/garlic/client/LegacyUsers;

    iget-object v1, p0, Lorg/garlic/client/LegacyUsers$7;->f$1:Ljava/lang/String;

    invoke-virtual {v0, v1, p1, p2}, Lorg/garlic/client/LegacyUsers;->lambda$options$7$org-garlic-client-LegacyUsers(Ljava/lang/String;Landroid/content/DialogInterface;I)V

    return-void
.end method
