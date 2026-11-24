.class public final synthetic Lorg/garlic/client/LegacyUsers$2;
.super Ljava/lang/Object;
.source "D8$$SyntheticClass"

# interfaces
.implements Ljava/lang/Runnable;


# annotations
.annotation runtime Lcom/android/tools/r8/annotations/LambdaMethod;
    holder = "Lorg/garlic/client/LegacyUsers;"
    method = "lambda$request$2"
    proto = "()V"
.end annotation


# instance fields
.field public final synthetic f$0:Lorg/garlic/client/LegacyUsers;


# direct methods
.method public synthetic constructor <init>(Lorg/garlic/client/LegacyUsers;)V
    .locals 0

    .line 0
    invoke-direct {p0}, Ljava/lang/Object;-><init>()V

    iput-object p1, p0, Lorg/garlic/client/LegacyUsers$2;->f$0:Lorg/garlic/client/LegacyUsers;

    return-void
.end method


# virtual methods
.method public final run()V
    .locals 1

    .line 0
    iget-object v0, p0, Lorg/garlic/client/LegacyUsers$2;->f$0:Lorg/garlic/client/LegacyUsers;

    invoke-virtual {v0}, Lorg/garlic/client/LegacyUsers;->lambda$request$2$org-garlic-client-LegacyUsers()V

    return-void
.end method
