.class public final synthetic Lorg/garlic/client/MainActivity$50;
.super Ljava/lang/Object;
.source "D8$$SyntheticClass"

# interfaces
.implements Landroid/content/DialogInterface$OnClickListener;


# annotations
.annotation runtime Lcom/android/tools/r8/annotations/LambdaMethod;
    holder = "Lorg/garlic/client/MainActivity;"
    method = "lambda$chooseApps$24"
    proto = "(Ljava/util/Set;Landroid/content/DialogInterface;I)V"
.end annotation


# instance fields
.field public final synthetic f$0:Lorg/garlic/client/MainActivity;

.field public final synthetic f$1:Ljava/util/Set;


# direct methods
.method public synthetic constructor <init>(Lorg/garlic/client/MainActivity;Ljava/util/Set;)V
    .locals 0

    .line 0
    invoke-direct {p0}, Ljava/lang/Object;-><init>()V

    iput-object p1, p0, Lorg/garlic/client/MainActivity$50;->f$0:Lorg/garlic/client/MainActivity;

    iput-object p2, p0, Lorg/garlic/client/MainActivity$50;->f$1:Ljava/util/Set;

    return-void
.end method


# virtual methods
.method public final onClick(Landroid/content/DialogInterface;I)V
    .locals 2

    .line 0
    iget-object v0, p0, Lorg/garlic/client/MainActivity$50;->f$0:Lorg/garlic/client/MainActivity;

    iget-object v1, p0, Lorg/garlic/client/MainActivity$50;->f$1:Ljava/util/Set;

    invoke-virtual {v0, v1, p1, p2}, Lorg/garlic/client/MainActivity;->lambda$chooseApps$24$org-garlic-client-MainActivity(Ljava/util/Set;Landroid/content/DialogInterface;I)V

    return-void
.end method
