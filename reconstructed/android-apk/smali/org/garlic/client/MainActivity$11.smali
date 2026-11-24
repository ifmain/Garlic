.class public final synthetic Lorg/garlic/client/MainActivity$11;
.super Ljava/lang/Object;
.source "D8$$SyntheticClass"

# interfaces
.implements Landroid/content/DialogInterface$OnClickListener;


# annotations
.annotation runtime Lcom/android/tools/r8/annotations/LambdaMethod;
    holder = "Lorg/garlic/client/MainActivity;"
    method = "lambda$userActions$47"
    proto = "(Ljava/lang/String;Ljava/lang/String;Landroid/content/DialogInterface;I)V"
.end annotation


# instance fields
.field public final synthetic f$0:Lorg/garlic/client/MainActivity;

.field public final synthetic f$1:Ljava/lang/String;

.field public final synthetic f$2:Ljava/lang/String;


# direct methods
.method public synthetic constructor <init>(Lorg/garlic/client/MainActivity;Ljava/lang/String;Ljava/lang/String;)V
    .locals 0

    .line 0
    invoke-direct {p0}, Ljava/lang/Object;-><init>()V

    iput-object p1, p0, Lorg/garlic/client/MainActivity$11;->f$0:Lorg/garlic/client/MainActivity;

    iput-object p2, p0, Lorg/garlic/client/MainActivity$11;->f$1:Ljava/lang/String;

    iput-object p3, p0, Lorg/garlic/client/MainActivity$11;->f$2:Ljava/lang/String;

    return-void
.end method


# virtual methods
.method public final onClick(Landroid/content/DialogInterface;I)V
    .locals 3

    .line 0
    iget-object v0, p0, Lorg/garlic/client/MainActivity$11;->f$0:Lorg/garlic/client/MainActivity;

    iget-object v1, p0, Lorg/garlic/client/MainActivity$11;->f$1:Ljava/lang/String;

    iget-object v2, p0, Lorg/garlic/client/MainActivity$11;->f$2:Ljava/lang/String;

    invoke-virtual {v0, v1, v2, p1, p2}, Lorg/garlic/client/MainActivity;->lambda$userActions$47$org-garlic-client-MainActivity(Ljava/lang/String;Ljava/lang/String;Landroid/content/DialogInterface;I)V

    return-void
.end method
