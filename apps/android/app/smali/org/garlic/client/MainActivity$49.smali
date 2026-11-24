.class public final synthetic Lorg/garlic/client/MainActivity$49;
.super Ljava/lang/Object;
.source "D8$$SyntheticClass"

# interfaces
.implements Landroid/content/DialogInterface$OnMultiChoiceClickListener;


# annotations
.annotation runtime Lcom/android/tools/r8/annotations/LambdaMethod;
    holder = "Lorg/garlic/client/MainActivity;"
    method = "lambda$chooseApps$23"
    proto = "(Ljava/util/Set;[Ljava/lang/String;Landroid/content/DialogInterface;IZ)V"
.end annotation


# instance fields
.field public final synthetic f$0:Ljava/util/Set;

.field public final synthetic f$1:[Ljava/lang/String;


# direct methods
.method public synthetic constructor <init>(Ljava/util/Set;[Ljava/lang/String;)V
    .locals 0

    .line 0
    invoke-direct {p0}, Ljava/lang/Object;-><init>()V

    iput-object p1, p0, Lorg/garlic/client/MainActivity$49;->f$0:Ljava/util/Set;

    iput-object p2, p0, Lorg/garlic/client/MainActivity$49;->f$1:[Ljava/lang/String;

    return-void
.end method


# virtual methods
.method public final onClick(Landroid/content/DialogInterface;IZ)V
    .locals 2

    .line 0
    iget-object v0, p0, Lorg/garlic/client/MainActivity$49;->f$0:Ljava/util/Set;

    iget-object v1, p0, Lorg/garlic/client/MainActivity$49;->f$1:[Ljava/lang/String;

    invoke-static {v0, v1, p1, p2, p3}, Lorg/garlic/client/MainActivity;->lambda$chooseApps$23(Ljava/util/Set;[Ljava/lang/String;Landroid/content/DialogInterface;IZ)V

    return-void
.end method
