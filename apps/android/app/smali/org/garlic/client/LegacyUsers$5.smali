.class public final synthetic Lorg/garlic/client/LegacyUsers$5;
.super Ljava/lang/Object;
.source "D8$$SyntheticClass"

# interfaces
.implements Landroid/content/DialogInterface$OnClickListener;


# annotations
.annotation runtime Lcom/android/tools/r8/annotations/LambdaMethod;
    holder = "Lorg/garlic/client/LegacyUsers;"
    method = "lambda$showList$5"
    proto = "(Landroid/content/DialogInterface;I)V"
.end annotation


# instance fields
.field public final synthetic f$0:Lorg/garlic/client/LegacyUsers;


# direct methods
.method public synthetic constructor <init>(Lorg/garlic/client/LegacyUsers;)V
    .locals 0

    .line 0
    invoke-direct {p0}, Ljava/lang/Object;-><init>()V

    iput-object p1, p0, Lorg/garlic/client/LegacyUsers$5;->f$0:Lorg/garlic/client/LegacyUsers;

    return-void
.end method


# virtual methods
.method public final onClick(Landroid/content/DialogInterface;I)V
    .locals 1

    .line 0
    iget-object v0, p0, Lorg/garlic/client/LegacyUsers$5;->f$0:Lorg/garlic/client/LegacyUsers;

    invoke-virtual {v0, p1, p2}, Lorg/garlic/client/LegacyUsers;->lambda$showList$5$org-garlic-client-LegacyUsers(Landroid/content/DialogInterface;I)V

    return-void
.end method
